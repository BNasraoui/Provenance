use super::*;

#[test]
fn output_replacement_race_preserves_the_intervening_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let original = utf8(temp.path().join("original-wiki"));
    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |_stage| {
            std::fs::rename(&output, &original)?;
            std::fs::create_dir(&output)?;
            std::fs::copy(
                original.join(OWNERSHIP_MANIFEST),
                output.join(OWNERSHIP_MANIFEST),
            )?;
            std::fs::write(output.join("caller.txt"), "keep me")?;
            Ok(())
        },
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::OutputChanged { .. }));
    assert_eq!(
        std::fs::read_to_string(output.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert!(original.join("index.html").is_file());
    assert!(!artifact(&output, "stage").exists());
}

#[test]
fn a_returned_install_failure_restores_the_previous_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();
    let mut rename_count = 0;

    let error = replace_output_with(
        &output,
        &paths,
        |_, _| {
            rename_count += 1;
            if rename_count == 2 {
                Err(std::io::Error::other("injected second rename failure"))
            } else {
                Ok(())
            }
        },
        |_| Ok(()),
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::ReplacementRolledBack { .. }));
    assert_eq!(
        std::fs::read_to_string(output.join("generation")).unwrap(),
        "old"
    );
    assert_eq!(
        std::fs::read_to_string(paths.stage.join("generation")).unwrap(),
        "new"
    );
    assert!(!paths.backup.exists());
}

#[test]
fn output_appearance_between_renames_preserves_every_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();
    let mut rename_count = 0;

    let error = replace_output_with(
        &output,
        &paths,
        |_, _| {
            rename_count += 1;
            if rename_count == 2 {
                std::fs::create_dir(&output)?;
            }
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::RollbackFailed { .. }));
    assert!(output.is_dir());
    assert_eq!(
        std::fs::read_to_string(paths.backup.join("generation")).unwrap(),
        "old"
    );
    assert_eq!(
        std::fs::read_to_string(paths.stage.join("generation")).unwrap(),
        "new"
    );
}

#[test]
fn committed_cleanup_failure_is_a_warning_not_a_returned_failure() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();

    let warnings = replace_output_with(
        &output,
        &paths,
        |_, _| Ok(()),
        |_path| Err(std::io::Error::other("injected cleanup failure")),
    )
    .unwrap();

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        std::fs::read_to_string(output.join("generation")).unwrap(),
        "new"
    );
    assert_eq!(
        std::fs::read_to_string(paths.backup.join("generation")).unwrap(),
        "old"
    );
}

#[test]
fn backup_cleanup_race_preserves_the_replacement_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    let displaced_backup = utf8(temp.path().join("displaced-backup"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();

    let warnings = replace_output_with(
        &output,
        &paths,
        |_, _| Ok(()),
        |path| {
            std::fs::rename(path, &displaced_backup)?;
            std::fs::create_dir(path)?;
            std::fs::write(path.join("caller"), "keep me")
        },
    )
    .unwrap();

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        std::fs::read_to_string(paths.backup.join("caller")).unwrap(),
        "keep me"
    );
    assert_eq!(
        std::fs::read_to_string(displaced_backup.join("generation")).unwrap(),
        "old"
    );
}
