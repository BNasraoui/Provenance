use super::*;

#[test]
fn publishes_a_complete_fresh_custom_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = Utf8PathBuf::from_path_buf(temp.path().join("wiki")).unwrap();

    let report = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    assert_eq!(report.page_count, 1);
    assert!(output.join("index.html").is_file());
    assert!(output.join("assets/provenance-wiki.css").is_file());
    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
    assert_no_transaction_artifacts(&output);
}

#[test]
fn adopts_an_empty_custom_directory() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();

    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
    assert_no_transaction_artifacts(&output);
}

#[test]
fn refuses_a_nonempty_unrecognized_custom_directory_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("caller.txt"), "keep me").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::CustomOutputUnrecognized { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(output.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn upgrades_a_recognized_custom_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();
    std::fs::write(output.join("stale-generated-file"), "old").unwrap();

    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    assert!(!output.join("stale-generated-file").exists());
    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
}

#[test]
fn repeatedly_replaces_a_recognized_output_without_leaving_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));

    for _ in 0..3 {
        let report = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

        assert_eq!(report.status, "ok");
        assert_no_transaction_artifacts(&output);
    }
}

#[test]
fn default_output_is_generator_owned_even_before_manifests() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("legacy-index.html"), "legacy").unwrap();

    publish(
        &empty_corpus(),
        PublicationOutput::generator_owned(output.clone()),
    )
    .unwrap();

    assert!(!output.join("legacy-index.html").exists());
    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
}

#[test]
fn rejects_an_unknown_manifest_version_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(
        output.join(OWNERSHIP_MANIFEST),
        r#"{"generator":"provenance-wiki","version":99}"#,
    )
    .unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::UnknownManifestVersion { version: 99, .. }
    ));
    assert_eq!(
        std::fs::read_to_string(output.join(OWNERSHIP_MANIFEST)).unwrap(),
        r#"{"generator":"provenance-wiki","version":99}"#
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn rejects_a_malformed_manifest_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join(OWNERSHIP_MANIFEST), "not json").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::InvalidManifest { .. }));
    assert_eq!(
        std::fs::read_to_string(output.join(OWNERSHIP_MANIFEST)).unwrap(),
        "not json"
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn rejects_an_oversized_manifest_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    let marker = vec![b' '; 64 * 1024];
    std::fs::write(output.join(OWNERSHIP_MANIFEST), &marker).unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::InvalidManifest { detail, .. } if detail.contains("too large")
    ));
    assert_eq!(
        std::fs::read(output.join(OWNERSHIP_MANIFEST)).unwrap(),
        marker
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn rejects_a_non_directory_output_root_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::write(&output, "caller bytes").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::OutputNotDirectory { .. }));
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "caller bytes");
    assert_no_transaction_artifacts(&output);
}

#[cfg(unix)]
#[test]
fn rejects_a_symlink_output_root_without_mutation() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let target = utf8(temp.path().join("target"));
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("caller.txt"), "keep me").unwrap();
    symlink(&target, &output).unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::OutputSymlink { .. }));
    assert!(output.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(
        std::fs::read_to_string(target.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert_no_transaction_artifacts(&output);
}

#[cfg(windows)]
#[test]
fn rejects_an_output_root_reparse_point_without_mutation() {
    use std::os::windows::fs::symlink_dir;

    let temp = tempfile::tempdir().unwrap();
    let target = utf8(temp.path().join("target"));
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("caller.txt"), "keep me").unwrap();
    symlink_dir(&target, &output).unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::OutputSymlink { .. }));
    assert_eq!(
        std::fs::read_to_string(target.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn does_not_infer_ownership_from_a_parseable_journal_or_nonce_names() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let journal = utf8(temp.path().join(".wiki.publish.json"));
    let nonce_stage = utf8(
        temp.path()
            .join(".wiki.stage.0123456789abcdef0123456789abcdef"),
    );
    let nonce_backup = utf8(
        temp.path()
            .join(".wiki.backup.0123456789abcdef0123456789abcdef"),
    );
    std::fs::write(
        &journal,
        "provenance-wiki-publication-v1\noutput=wiki\nnonce=0123456789abcdef0123456789abcdef\n",
    )
    .unwrap();
    std::fs::create_dir(&nonce_stage).unwrap();
    std::fs::create_dir(&nonce_backup).unwrap();
    std::fs::write(nonce_stage.join("caller"), "stage").unwrap();
    std::fs::write(nonce_backup.join("caller"), "backup").unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("caller"), "live").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::CustomOutputUnrecognized { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(output.join("caller")).unwrap(),
        "live"
    );
    assert!(journal.is_file());
    assert_eq!(
        std::fs::read_to_string(nonce_stage.join("caller")).unwrap(),
        "stage"
    );
    assert_eq!(
        std::fs::read_to_string(nonce_backup.join("caller")).unwrap(),
        "backup"
    );
}
