use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use assert_cmd::Command;
use provenance_macros::verifies;
use serde_json::json;

fn provenance() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("provenance"))
}

fn check_from(directory: &Path, input: &[u8]) -> std::process::Output {
    provenance()
        .current_dir(directory)
        .args(["sdk", "check-statement", "--format", "json"])
        .write_stdin(input)
        .output()
        .unwrap()
}

fn expected_json(statement: &str) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(&provenance_ste100::check_descriptive(statement))
        .expect("the authoritative report serializes");
    bytes.push(b'\n');
    bytes
}

fn file_bytes(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries = std::fs::read_dir(directory)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
#[verifies("rule_ste_sdk_statement_report", examples)]
fn check_statement_returns_the_authoritative_report_for_clean_and_flagged_text() {
    let outside_repo = tempfile::tempdir().unwrap();
    let long_sentence = format!("{}.", vec!["word"; 26].join(" "));
    for statement in [
        "Install the cover.",
        "Stop; wait.",
        "A; B;; C;",
        "It isn't ready.",
        &long_sentence,
    ] {
        let output = check_from(
            outside_repo.path(),
            &serde_json::to_vec(&json!({"statement": statement})).unwrap(),
        );

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected_json(statement));
        assert!(output.stderr.is_empty());
    }
}

#[test]
#[verifies("rule_ste_sdk_statement_report", examples)]
fn check_statement_preserves_exact_unicode_byte_spans() {
    let outside_repo = tempfile::tempdir().unwrap();
    let output = check_from(outside_repo.path(), r#"{"statement":"é; 文;"}"#.as_bytes());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(report["findings"][0]["span"], json!({"start": 2, "end": 3}));
    assert_eq!(report["findings"][1]["span"], json!({"start": 7, "end": 8}));
}

#[test]
#[verifies("rule_ste_sdk_statement_report", conformance)]
fn check_statement_json_is_stable_and_byte_identical_to_direct_checker_output() {
    let outside_repo = tempfile::tempdir().unwrap();
    let input = r#"{"statement":"é; Stop;;"}"#.as_bytes();
    let first = check_from(outside_repo.path(), input);
    let second = check_from(outside_repo.path(), input);

    assert!(first.status.success());
    assert_eq!(first.stdout, expected_json("é; Stop;;"));
    assert_eq!(second.stdout, first.stdout);
}

#[test]
#[verifies("rule_ste_sdk_statement_request_schema", examples)]
fn check_statement_rejects_malformed_missing_and_unknown_json_fields() {
    let outside_repo = tempfile::tempdir().unwrap();
    for (input, expected_error) in [
        (b"{".as_slice(), "EOF while parsing"),
        (br"{}", "missing field `statement`"),
        (
            br#"{"statement":"Install the cover.","extra":true}"#,
            "unknown field `extra`",
        ),
    ] {
        let output = check_from(outside_repo.path(), input);

        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "expected {expected_error:?} in {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[verifies("rule_ste_sdk_statement_repository_independence", examples)]
fn check_statement_works_outside_a_repository_and_writes_nothing() {
    let outside_repo = tempfile::tempdir().unwrap();
    let before = file_bytes(outside_repo.path());

    let output = check_from(
        outside_repo.path(),
        br#"{"statement":"Install the cover."}"#,
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(file_bytes(outside_repo.path()), before);
}

#[test]
#[verifies("rule_ste_sdk_statement_repository_independence", examples)]
fn check_statement_leaves_an_existing_repository_byte_for_byte_unchanged() {
    let repository = tempfile::tempdir().unwrap();
    provenance()
        .args([
            "init",
            "--path",
            repository.path().to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();
    std::fs::write(repository.path().join("requirement.txt"), "Stop; wait.").unwrap();
    let before = file_bytes(repository.path());

    let output = check_from(repository.path(), br#"{"statement":"Stop; wait."}"#);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(file_bytes(repository.path()), before);
}

#[test]
fn check_statement_has_a_generous_interactive_latency_budget() {
    let outside_repo = tempfile::tempdir().unwrap();
    let started = Instant::now();
    for _ in 0..5 {
        let output = check_from(outside_repo.path(), br#"{"statement":"Stop; wait."}"#);
        assert!(output.status.success());
    }

    // Ten seconds permits slow shared CI while bounding five real process round trips.
    // It demonstrates an interactive adapter budget, not checker microbenchmark speed.
    assert!(started.elapsed() < Duration::from_secs(10));
}
