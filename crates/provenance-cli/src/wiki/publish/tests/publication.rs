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
