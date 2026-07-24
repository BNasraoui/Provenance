use super::*;

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
fn stage_creation_race_preserves_the_foreign_tree() {
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

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::Io { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(stage.join("caller")).unwrap(),
        "keep me"
    );
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

    let report = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |_| {
            std::fs::rename(&parent, &displaced_parent)?;
            std::fs::create_dir(&parent)?;
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(report.status, "ok");
    assert!(displaced_parent.join("wiki/index.html").is_file());
    assert!(!output.exists());
    assert!(std::fs::read_dir(&parent).unwrap().next().is_none());
}
