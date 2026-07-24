use super::*;

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
