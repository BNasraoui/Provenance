use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;

fn git(repo: &Path, args: &[&str]) -> String {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn write(repo: &Path, relative: &str, contents: &str) {
    let path = repo.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn commit(repo: &Path, message: &str, date: Option<&str>) -> String {
    git(repo, &["add", "."]);
    let mut command = ProcessCommand::new("git");
    command.arg("-C").arg(repo).args(["commit", "-m", message]);
    if let Some(date) = date {
        command
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    git(repo, &["rev-parse", "HEAD"])
}

fn review(repo: &Path, args: &[&str]) -> Value {
    let mut command = Command::cargo_bin("provenance").unwrap();
    command.args([
        "evidence-review",
        "--repo",
        repo.to_str().unwrap(),
        "--scope",
        "default",
        "--format",
        "json",
    ]);
    command.args(args);
    let output = command.assert().success().get_output().stdout.clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn base_override_controls_diff_and_age_without_rewriting_source_revision() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            dir.path().to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();

    write(
        dir.path(),
        "src/pinned.rs",
        "const PINNED: &str = \"old\";\n",
    );
    write(
        dir.path(),
        "src/unpinned.rs",
        "const UNPINNED: &str = \"old\";\n",
    );
    let explicit_base = commit(
        dir.path(),
        "old comparison base",
        Some("2000-01-01T00:00:00Z"),
    );

    write(
        dir.path(),
        "src/pinned.rs",
        "const PINNED: &str = \"new\";\n",
    );
    let source_pin = commit(dir.path(), "fresh source pin", None);
    write(
        dir.path(),
        "src/unpinned.rs",
        "const UNPINNED: &str = \"new\";\n",
    );
    let head = commit(dir.path(), "candidate", None);
    assert_ne!(source_pin, explicit_base);

    write(
        dir.path(),
        ".provenance/state/scopes/default/sources/source.jsonl",
        &format!(
            "{{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"source_pinned\",\"name\":\"Pinned\",\"source_type\":\"system_state\",\"commit_pin\":\"{source_pin}\"}}\n{{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"source_unpinned\",\"name\":\"Unpinned\",\"source_type\":\"system_state\"}}\n"
        ),
    );
    write(
        dir.path(),
        ".provenance/state/scopes/default/ideation/proposal_cards.jsonl",
        r#"{"schema_version":1,"scope_id":"default","id":"proposal_pinned","proposal_key":"backtrace/pinned","proposal_type":"requirement_candidate","title":"Pinned evidence","summary":"candidate","traceability":{"target":{"artifact_type":"source","artifact_id":"source_pinned"},"source_ids":["source_pinned"],"evidence_references":[{"reference_id":"ev_pinned","evidence_type":"artifact","summary":"pinned line","file_path":"src/pinned.rs","line":1}],"supporting_claim_ids":[]},"promotion_state":"proposed"}
{"schema_version":1,"scope_id":"default","id":"proposal_unpinned","proposal_key":"backtrace/unpinned","proposal_type":"requirement_candidate","title":"Unpinned evidence","summary":"candidate","traceability":{"target":{"artifact_type":"source","artifact_id":"source_unpinned"},"source_ids":["source_unpinned"],"evidence_references":[{"reference_id":"ev_unpinned","evidence_type":"artifact","summary":"unpinned line","file_path":"src/unpinned.rs","line":1}],"supporting_claim_ids":[]},"promotion_state":"proposed"}
"#,
    );

    let report = review(
        dir.path(),
        &[
            "--base",
            &explicit_base,
            "--head",
            &head,
            "--min-age-days",
            "1",
        ],
    );

    assert_eq!(report["diffs"][0]["base"], explicit_base);
    assert_eq!(report["diffs"][0]["head"], head);
    assert_eq!(report["summary"]["evidence_reverified"], 2);
    assert_eq!(report["evidence"][0]["source_revision"], source_pin);
    assert_eq!(report["evidence"][0]["base_revision"], explicit_base);
    assert_eq!(report["evidence"][1]["source_revision"], Value::Null);
    assert_eq!(report["evidence"][1]["base_revision"], explicit_base);
}
