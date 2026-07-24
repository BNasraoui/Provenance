use super::*;

#[cfg(unix)]
#[test]
fn rejects_a_symlink_lock_without_following_it() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let lock = artifact(&output, "lock");
    let target = utf8(temp.path().join("caller.txt"));
    std::fs::write(&target, "keep me").unwrap();
    symlink(&target, &lock).unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::UnsafeLockPath { .. }));
    assert!(lock.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "keep me");
    assert!(!output.exists());
}

#[test]
fn leaves_ambiguous_interruption_artifacts_untouched() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let stage = artifact(&output, "stage");
    let backup = artifact(&output, "backup");
    std::fs::create_dir(&stage).unwrap();
    std::fs::create_dir(&backup).unwrap();
    std::fs::write(stage.join("unknown"), "stage bytes").unwrap();
    std::fs::write(backup.join("unknown"), "backup bytes").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::AmbiguousArtifacts { .. }));
    assert_eq!(
        std::fs::read_to_string(stage.join("unknown")).unwrap(),
        "stage bytes"
    );
    assert_eq!(
        std::fs::read_to_string(backup.join("unknown")).unwrap(),
        "backup bytes"
    );
    assert!(!output.exists());
}

#[test]
fn staging_callback_failure_removes_the_owned_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let stage = artifact(&output, "stage");

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |path| {
            std::fs::write(path.join("caller"), "keep me").unwrap();
            Err(std::io::Error::other("injected staging failure"))
        },
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::Io { .. }));
    assert!(!stage.exists());
    assert!(!artifact(&output, "lock").exists());
    assert!(!output.exists());
}

#[test]
fn parent_replacement_does_not_redirect_the_transaction() {
    let temp = tempfile::tempdir().unwrap();
    let parent = utf8(temp.path().join("site"));
    let displaced_parent = utf8(temp.path().join("original-site"));
    let output = parent.join("wiki");
    std::fs::create_dir(&parent).unwrap();

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |_| {
            std::fs::rename(&parent, &displaced_parent)?;
            std::fs::create_dir(&parent)?;
            Ok(())
        },
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::OutputChanged { .. }));
    assert!(!displaced_parent.join("wiki").exists());
    assert!(!output.exists());
    assert!(std::fs::read_dir(&parent).unwrap().next().is_none());
}

#[cfg(unix)]
#[test]
fn staged_child_symlink_cannot_redirect_generated_writes() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let external = utf8(temp.path().join("external"));
    std::fs::create_dir(&external).unwrap();

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| symlink(&external, stage.join("assets")),
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::Io { .. }));
    assert!(!external.join("provenance-wiki.css").exists());
    assert!(!artifact(&output, "stage").exists());
    assert!(!output.exists());
}

#[cfg(windows)]
#[test]
fn staged_child_reparse_point_cannot_redirect_generated_writes() {
    use std::os::windows::fs::symlink_dir;

    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let external = utf8(temp.path().join("external"));
    std::fs::create_dir(&external).unwrap();

    let result = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| symlink_dir(&external, stage.join("assets")),
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("publication followed a staged reparse point"),
    };

    assert!(matches!(error, PublishError::Io { .. }));
    assert!(!external.join("provenance-wiki.css").exists());
    assert!(!artifact(&output, "stage").exists());
    assert!(!output.exists());
}

#[test]
fn replaced_stage_is_not_installed() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let displaced = utf8(temp.path().join("displaced-stage"));

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| {
            std::fs::rename(stage, &displaced)?;
            std::fs::create_dir(stage)?;
            std::fs::write(stage.join("caller"), "keep me")
        },
    )
    .unwrap_err();

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::OutputChanged { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(artifact(&output, "stage").join("caller")).unwrap(),
        "keep me"
    );
    assert!(displaced.join("index.html").is_file());
    assert!(!output.exists());
}

#[test]
fn empty_replacement_stage_is_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let displaced = utf8(temp.path().join("displaced-stage"));
    let stage = artifact(&output, "stage");

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| {
            std::fs::rename(stage, &displaced)?;
            std::fs::create_dir(stage)
        },
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::CleanupFailed { .. }));
    assert!(stage.is_dir());
    assert!(std::fs::read_dir(stage).unwrap().next().is_none());
    assert!(displaced.join("index.html").is_file());
    assert!(!output.exists());
}

#[test]
fn stage_replaced_after_identity_check_is_not_installed() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    let displaced_stage = utf8(temp.path().join("displaced-stage"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "generated").unwrap();
    let mut rename_count = 0;

    let error = replace_output_with(
        &output,
        &paths,
        |_, _| {
            rename_count += 1;
            if rename_count == 1 {
                std::fs::rename(&paths.stage, &displaced_stage)?;
                std::fs::create_dir(&paths.stage)?;
                std::fs::write(paths.stage.join("generation"), "foreign")?;
            }
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::OutputChanged { .. }));
    assert_eq!(
        std::fs::read_to_string(output.join("generation")).unwrap(),
        "old"
    );
    assert_eq!(
        std::fs::read_to_string(paths.stage.join("generation")).unwrap(),
        "foreign"
    );
    assert_eq!(
        std::fs::read_to_string(displaced_stage.join("generation")).unwrap(),
        "generated"
    );
}

#[test]
fn lock_replacement_before_cleanup_is_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let lock = artifact(&output, "lock");
    let displaced_lock = utf8(temp.path().join("displaced-lock"));

    let report = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |_| {
            std::fs::rename(&lock, &displaced_lock)?;
            std::fs::write(&lock, "foreign lock")
        },
    )
    .unwrap();

    assert_eq!(report.status, "ok_with_cleanup_required");
    assert_eq!(std::fs::read_to_string(lock).unwrap(), "foreign lock");
    assert!(displaced_lock.is_file());
    assert!(!artifact(&output, "lock.cleanup").exists());
}
