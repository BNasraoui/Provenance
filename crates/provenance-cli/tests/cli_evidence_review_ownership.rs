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

#[test]
fn source_target_proposal_rejects_evidence_owned_by_a_different_source() {
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

    write(dir.path(), "src/guard.rs", "return Err(\"invalid\");\n");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "base"]);
    let base = git(dir.path(), &["rev-parse", "HEAD"]);
    write(
        dir.path(),
        ".provenance/state/scopes/default/sources/source.jsonl",
        &format!(
            "{{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"source_a\",\"name\":\"A\",\"source_type\":\"system_state\",\"commit_pin\":\"{base}\"}}\n{{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"source_b\",\"name\":\"B\",\"source_type\":\"system_state\",\"commit_pin\":\"{base}\"}}\n"
        ),
    );
    write(
        dir.path(),
        ".provenance/state/scopes/default/ideation/proposal_cards.jsonl",
        r#"{"schema_version":1,"scope_id":"default","id":"proposal_mismatch","proposal_key":"backtrace/mismatch","proposal_type":"requirement_candidate","title":"Reject invalid input","summary":"candidate","traceability":{"target":{"artifact_type":"source","artifact_id":"source_a"},"source_ids":["source_b"],"evidence_references":[{"reference_id":"ev_mismatch","evidence_type":"artifact","summary":"guard","file_path":"src/guard.rs","line":1}],"supporting_claim_ids":[]},"promotion_state":"proposed"}
"#,
    );
    write(dir.path(), "src/guard.rs", "return Ok(());\n");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "change"]);

    let output = Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "evidence-review",
            "--repo",
            dir.path().to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(report["summary"]["evidence_reverified"], 0);
    assert_eq!(report["evidence"], serde_json::json!([]));
    assert!(report["diagnostics"][0]
        .as_str()
        .unwrap()
        .contains("targets source source_a but its sole source_id is source_b"));
}
