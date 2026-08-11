use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use serde_json::{json, Value};

fn provenance() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("provenance"))
}

fn init_repo() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    provenance()
        .args([
            "init",
            "--path",
            directory.path().to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();
    directory
}

fn spec(statement: &str) -> Value {
    json!({
        "schema_version": 1,
        "declared_by": "spec://typescript/share-links",
        "sources": [{
            "key": "linear:ABC-123",
            "name": "Linear ABC-123",
            "kind": "linear",
            "url": "https://linear.app/example/issue/ABC-123"
        }],
        "requirements": [{
            "key": "sharing",
            "statement": "Users can securely share documentation",
            "sources": ["linear:ABC-123"]
        }],
        "rules": [{
            "key": "expiry",
            "requirement": "sharing",
            "statement": statement
        }]
    })
}

fn apply(repo: &str, input: &Value) -> Value {
    let output = provenance()
        .args([
            "sdk", "apply", "--repo", repo, "--scope", "default", "--format", "json",
        ])
        .write_stdin(serde_json::to_vec(input).unwrap())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "sdk apply failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn apply_materializes_typed_declarations_as_canonical_graph_records() {
    let directory = init_repo();
    let repo = directory.path().to_str().unwrap();

    let result = apply(repo, &spec("Share links expire within 30 days"));

    assert_eq!(result["created"], 3);
    assert_eq!(result["updated"], 0);
    assert_eq!(result["resources"][1]["key"], "sharing");
    assert_eq!(result["resources"][1]["id"], "sharing");
    assert_eq!(result["resources"][2]["key"], "expiry");
    assert_eq!(result["resources"][2]["id"], "expiry");
    assert!(result["resources"][0]["id"]
        .as_str()
        .unwrap()
        .starts_with("source_linear_abc-123_"));

    provenance()
        .args([
            "rules", "show", "--repo", repo, "--scope", "default", "--id", "expiry", "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("Share links expire within 30 days"))
        .stdout(contains("spec://typescript/share-links"));

    provenance()
        .args([
            "traceability",
            "expiry",
            "--repo",
            repo,
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("sharing"));

    let wiki = directory.path().join("wiki");
    provenance()
        .args([
            "wiki",
            "build",
            "--repo",
            repo,
            "--scope",
            "default",
            "--out",
            wiki.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success();
    let rule_page = std::fs::read_to_string(wiki.join("rules/expiry/index.html")).unwrap();
    assert!(rule_page.contains("Share links expire within 30 days"));

    provenance()
        .args(["check", "--repo", repo, "--format", "json"])
        .assert()
        .success();
}

#[test]
fn apply_updates_only_records_owned_by_the_same_spec() {
    let directory = init_repo();
    let repo = directory.path().to_str().unwrap();
    apply(repo, &spec("Share links expire within 30 days"));

    let result = apply(repo, &spec("Share links expire within 14 days"));

    assert_eq!(result["created"], 0);
    assert_eq!(result["updated"], 1);
    assert_eq!(result["unchanged"], 2);
    provenance()
        .args([
            "rules", "show", "--repo", repo, "--scope", "default", "--id", "expiry", "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("Share links expire within 14 days"));
}

#[test]
fn apply_refuses_to_take_over_an_unowned_record() {
    let directory = init_repo();
    let repo = directory.path().to_str().unwrap();
    provenance()
        .args([
            "requirements",
            "create",
            "--repo",
            repo,
            "--scope",
            "default",
            "--id",
            "sharing",
            "--statement",
            "Externally managed statement",
        ])
        .assert()
        .success();

    provenance()
        .args(["sdk", "apply", "--repo", repo, "--scope", "default"])
        .write_stdin(serde_json::to_vec(&spec("Share links expire")).unwrap())
        .assert()
        .failure()
        .stderr(contains("sharing").and(contains("not owned")));

    provenance()
        .args(["export", "--repo", repo, "--scope", "default"])
        .assert()
        .success()
        .stdout(contains("Externally managed statement"))
        .stdout(contains("Share links expire").not());
}

#[test]
fn verification_runs_are_linked_to_the_rule_and_record_the_outcome() {
    let directory = init_repo();
    let repo = directory.path().to_str().unwrap();
    apply(repo, &spec("Share links expire within 30 days"));

    let begun = provenance()
        .args([
            "sdk",
            "begin-verification",
            "--repo",
            repo,
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .write_stdin(
            serde_json::to_vec(&json!({
                "rule": "expiry",
                "method": "examples",
                "declared_by": "ci://node-test",
                "file": "share-links.test.ts",
                "symbol": "share links expire"
            }))
            .unwrap(),
        )
        .output()
        .unwrap();
    assert!(begun.status.success());
    let begun: Value = serde_json::from_slice(&begun.stdout).unwrap();
    let run = begun["id"].as_str().unwrap();

    provenance()
        .args([
            "sdk",
            "complete-verification",
            "--repo",
            repo,
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .write_stdin(
            serde_json::to_vec(&json!({
                "run": run,
                "status": "passed"
            }))
            .unwrap(),
        )
        .assert()
        .success()
        .stdout(contains("\"status\": \"passed\""));

    provenance()
        .args([
            "sdk",
            "verification-runs",
            "--repo",
            repo,
            "--scope",
            "default",
            "--rule",
            "expiry",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("ci://node-test"))
        .stdout(contains("share-links.test.ts"))
        .stdout(contains("\"status\": \"passed\""));
}

#[test]
fn verification_cannot_begin_for_an_unknown_rule() {
    let directory = init_repo();
    let repo = directory.path().to_str().unwrap();

    provenance()
        .args([
            "sdk",
            "begin-verification",
            "--repo",
            repo,
            "--scope",
            "default",
        ])
        .write_stdin(
            serde_json::to_vec(&json!({
                "rule": "missing",
                "method": "examples",
                "declared_by": "ci://node-test"
            }))
            .unwrap(),
        )
        .assert()
        .failure()
        .stderr(contains("missing").and(contains("does not exist")));
}
