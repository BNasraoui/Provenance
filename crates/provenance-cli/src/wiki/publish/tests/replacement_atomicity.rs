use super::*;

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

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::OutputChanged { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(output.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert!(original.join("index.html").is_file());
    assert!(artifact(&output, "stage").is_dir());
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
