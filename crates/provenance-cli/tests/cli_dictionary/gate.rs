use crate::support::{
    create_requirement, error_json, imported_dictionary, init, write_reference, UNAPPROVED_WORD,
};
use provenance_macros::verifies;
use provenance_ste100::store_dictionary_index;

fn statement() -> String {
    format!("The {UNAPPROVED_WORD} item stops.")
}

#[test]
#[verifies("rule_ste_dictionary_unapproved_word", examples)]
#[verifies("rule_ste_dictionary_reference_resolution", examples)]
fn a_reference_with_a_loadable_index_rejects_an_unapproved_word() {
    let scratch = tempfile::tempdir().unwrap();
    let repo = scratch.path().join("repo");
    let index_directory = scratch.path().join("index");
    std::fs::create_dir_all(&repo).unwrap();
    init(&repo);
    let dictionary = imported_dictionary();
    store_dictionary_index(dictionary, &index_directory).unwrap();
    write_reference(&repo, dictionary);

    let output = create_requirement(&repo, &index_directory, "req_gate", &statement());

    assert!(
        !output.status.success(),
        "the write must fail: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let error = error_json(&output);
    assert_eq!(error["field"], "statement");
    let findings = error["findings"]
        .as_array()
        .expect("the error lists findings");
    assert_eq!(findings.len(), 1, "findings: {findings:?}");
    assert_eq!(findings[0]["rule"], "1.1");
    assert_eq!(findings[0]["kind"], "violation");
    let span = &findings[0]["span"];
    let start = usize::try_from(span["start"].as_u64().unwrap()).unwrap();
    let end = usize::try_from(span["end"].as_u64().unwrap()).unwrap();
    assert_eq!(&statement()[start..end], UNAPPROVED_WORD);
}

#[test]
#[verifies("rule_ste_dictionary_reference_resolution", examples)]
fn a_project_without_a_reference_accepts_the_same_statement() {
    let scratch = tempfile::tempdir().unwrap();
    let repo = scratch.path().join("repo");
    let index_directory = scratch.path().join("index");
    std::fs::create_dir_all(&repo).unwrap();
    init(&repo);
    store_dictionary_index(imported_dictionary(), &index_directory).unwrap();

    let output = create_requirement(&repo, &index_directory, "req_gate", &statement());

    assert!(
        output.status.success(),
        "the write must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[verifies("rule_ste_dictionary_reference_resolution", examples)]
fn a_reference_without_an_index_accepts_the_same_statement() {
    let scratch = tempfile::tempdir().unwrap();
    let repo = scratch.path().join("repo");
    let index_directory = scratch.path().join("index");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&index_directory).unwrap();
    init(&repo);
    write_reference(&repo, imported_dictionary());

    let output = create_requirement(&repo, &index_directory, "req_gate", &statement());

    assert!(
        output.status.success(),
        "a missing index must fall back to the data-free checks: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
