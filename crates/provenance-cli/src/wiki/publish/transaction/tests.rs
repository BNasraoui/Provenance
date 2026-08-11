use super::*;

#[test]
fn rollback_failure_leaves_every_ambiguous_tree_for_operator_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let output = Utf8PathBuf::from_path_buf(temp.path().join("wiki")).unwrap();
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
            if rename_count > 1 {
                Err(std::io::Error::other("injected rename failure"))
            } else {
                Ok(())
            }
        },
        |_| Ok(()),
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::RollbackFailed { .. }));
    assert!(!output.exists());
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
fn no_replace_rename_preserves_an_output_that_appeared() {
    let temp = tempfile::tempdir().unwrap();
    let output = Utf8PathBuf::from_path_buf(temp.path().join("wiki")).unwrap();
    let stage = Utf8PathBuf::from_path_buf(temp.path().join("stage")).unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("caller"), "keep me").unwrap();
    std::fs::create_dir(&stage).unwrap();
    std::fs::write(stage.join("generated"), "new").unwrap();

    let parent = ownership::open_directory_no_follow(temp.path()).unwrap();
    let error =
        replacement::rename_no_replace_at(&parent, output.parent().unwrap(), "stage", "wiki")
            .unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(
        std::fs::read_to_string(output.join("caller")).unwrap(),
        "keep me"
    );
    assert_eq!(
        std::fs::read_to_string(stage.join("generated")).unwrap(),
        "new"
    );
}

#[cfg(windows)]
#[test]
fn no_replace_rename_anchors_its_destination_to_the_parent_handle() {
    use std::os::windows::fs::symlink_dir;

    let temp = tempfile::tempdir().unwrap();
    let parent_path = Utf8PathBuf::from_path_buf(temp.path().join("site")).unwrap();
    let displaced_parent = Utf8PathBuf::from_path_buf(temp.path().join("original-site")).unwrap();
    let external = Utf8PathBuf::from_path_buf(temp.path().join("external")).unwrap();
    std::fs::create_dir(&parent_path).unwrap();
    std::fs::create_dir(parent_path.join("stage")).unwrap();
    std::fs::write(parent_path.join("stage/generated"), "new").unwrap();
    std::fs::create_dir(&external).unwrap();
    let parent = ownership::open_directory_no_follow(parent_path.as_std_path()).unwrap();
    std::fs::rename(&parent_path, &displaced_parent).unwrap();
    symlink_dir(&external, &parent_path).unwrap();

    replacement::rename_no_replace_at(&parent, &parent_path, "stage", "wiki").unwrap();

    assert_eq!(
        std::fs::read_to_string(displaced_parent.join("wiki/generated")).unwrap(),
        "new"
    );
    assert!(!displaced_parent.join("stage").exists());
    assert!(!external.join("wiki").exists());
}
