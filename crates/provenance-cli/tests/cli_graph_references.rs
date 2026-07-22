use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::path::Path;

fn provenance(repo: &Path) -> Command {
    let mut command = Command::cargo_bin("provenance").unwrap();
    command.args(["--quiet"]);
    command.current_dir(repo);
    command
}

fn git(repo: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

fn committed_store() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(
        temp.path(),
        &["config", "user.name", "Graph Reference Test"],
    );
    git(
        temp.path(),
        &["config", "user.email", "graph-reference@example.invalid"],
    );
    provenance(temp.path())
        .args(["init", "--path", ".", "--scope", "default"])
        .assert()
        .success();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "canonical graph"]);
    temp
}

fn issue(repo: &Path, extra: &[&str]) -> Value {
    let output = provenance(repo)
        .args([
            "graph-reference",
            "issue",
            "--repo",
            ".",
            "--scope",
            "default",
        ])
        .args(extra)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

fn write_reference(repo: &Path, reference: &Value) -> std::path::PathBuf {
    let path = repo.join("reference.json");
    std::fs::write(&path, serde_json::to_vec(reference).unwrap()).unwrap();
    path
}

#[test]
fn issue_is_versioned_deterministic_and_correlation_is_not_identity() {
    let temp = committed_store();
    let first = issue(temp.path(), &[]);
    let second = issue(
        temp.path(),
        &[
            "--correlation-system",
            "github",
            "--correlation-key",
            "owner/repo#42",
        ],
    );

    assert_eq!(first["schema_version"], 1);
    assert_eq!(first["reference_id"], second["reference_id"]);
    assert_eq!(first["graph_digest"], second["graph_digest"]);
    assert_eq!(second["correlation"]["system"], "github");
    assert_eq!(second["correlation"]["key"], "owner/repo#42");
    assert_eq!(first["commit"].as_str().unwrap().len(), 40);
}

#[test]
fn implicit_head_requires_only_relevant_canonical_state_to_be_clean() {
    let temp = committed_store();
    std::fs::write(temp.path().join("unrelated.txt"), "dirty but irrelevant\n").unwrap();
    issue(temp.path(), &[]);

    let manifest = temp.path().join(".provenance/state/manifest.json");
    let original = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(&manifest, original.replace("\".\"", "\"src\"")).unwrap();
    provenance(temp.path())
        .args([
            "graph-reference",
            "issue",
            "--repo",
            ".",
            "--scope",
            "default",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("incomplete"));

    std::fs::write(&manifest, original.replace("\".\"", "\"staged\"")).unwrap();
    git(temp.path(), &["add", ".provenance/state/manifest.json"]);
    std::fs::write(&manifest, &original).unwrap();
    provenance(temp.path())
        .args([
            "graph-reference",
            "issue",
            "--repo",
            ".",
            "--scope",
            "default",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("incomplete"));
}

#[test]
fn explicit_commit_allows_dirty_state_and_exact_operations_read_the_pin() {
    let temp = committed_store();
    let reference = issue(temp.path(), &[]);
    let reference_path = write_reference(temp.path(), &reference);
    let manifest = temp.path().join(".provenance/state/manifest.json");
    std::fs::write(&manifest, "not the pinned manifest\n").unwrap();

    for operation in ["show", "verify", "exact-export"] {
        provenance(temp.path())
            .args([
                "graph-reference",
                operation,
                "--repo",
                ".",
                "--reference",
                reference_path.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"schema_version\": 1"));
    }
}

#[test]
fn verify_reports_typed_mismatch_and_missing_errors() {
    let temp = committed_store();
    let mut reference = issue(temp.path(), &[]);
    reference["graph_digest"] = Value::String(format!("sha256:{}", "0".repeat(64)));
    let reference_path = write_reference(temp.path(), &reference);
    provenance(temp.path())
        .args([
            "graph-reference",
            "verify",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mismatched"));

    reference["commit"] = Value::String("0".repeat(40));
    let reference_path = write_reference(temp.path(), &reference);
    provenance(temp.path())
        .args([
            "graph-reference",
            "show",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing"));
}

#[test]
fn incomplete_reference_and_workflow_specific_fields_are_rejected() {
    let temp = committed_store();
    let mut reference = issue(temp.path(), &[]);
    reference["workflow_id"] = Value::String("workflowd-123".into());
    let reference_path = write_reference(temp.path(), &reference);
    provenance(temp.path())
        .args([
            "graph-reference",
            "verify",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("incomplete"));

    let mut reference = issue(temp.path(), &[]);
    reference.as_object_mut().unwrap().remove("graph_digest");
    let reference_path = write_reference(temp.path(), &reference);
    provenance(temp.path())
        .args([
            "graph-reference",
            "verify",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("incomplete"));
}

#[test]
fn identity_changes_by_commit_and_graph_while_digest_tracks_graph_only() {
    let temp = committed_store();
    let empty = issue(temp.path(), &[]);
    git(
        temp.path(),
        &["commit", "--allow-empty", "-qm", "new handoff commit"],
    );
    let same_graph = issue(temp.path(), &[]);
    assert_eq!(empty["graph_digest"], same_graph["graph_digest"]);
    assert_ne!(empty["reference_id"], same_graph["reference_id"]);

    provenance(temp.path())
        .args([
            "requirements",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "req_pinned",
            "--statement",
            "Pinned graph content",
        ])
        .assert()
        .success();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "change canonical graph"]);
    let changed_graph = issue(temp.path(), &[]);
    assert_ne!(same_graph["graph_digest"], changed_graph["graph_digest"]);
    assert_ne!(same_graph["reference_id"], changed_graph["reference_id"]);

    let old_reference = write_reference(temp.path(), &same_graph);
    let output = provenance(temp.path())
        .args([
            "graph-reference",
            "exact-export",
            "--repo",
            ".",
            "--reference",
            old_reference.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(export["graph"]["requirements"], serde_json::json!([]));
}

#[test]
fn graph_reference_schemas_are_exposed() {
    let temp = committed_store();
    for artifact in ["graph-reference", "graph-reference-export"] {
        provenance(temp.path())
            .args(["schema", "show", artifact, "--format", "json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"const\": 1"))
            .stdout(predicate::str::contains("additionalProperties"));
    }
}

#[test]
fn emitted_graph_reference_schema_validates_an_issued_reference() {
    let temp = committed_store();
    let reference = issue(temp.path(), &[]);
    let output = provenance(temp.path())
        .args(["schema", "show", "graph-reference", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let shown: Value = serde_json::from_slice(&output).unwrap();
    let schema = shown.get("schema").unwrap();
    let validator = jsonschema::JSONSchema::compile(schema).unwrap();

    assert!(validator.is_valid(&reference));
}

#[test]
fn explicit_commit_issues_from_pin_despite_relevant_staged_and_worktree_changes() {
    let temp = committed_store();
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let head = String::from_utf8(head.stdout).unwrap().trim().to_string();
    let manifest = temp.path().join(".provenance/state/manifest.json");
    let original = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(&manifest, original.replace("\".\"", "\"staged\"")).unwrap();
    git(temp.path(), &["add", ".provenance/state/manifest.json"]);
    std::fs::write(&manifest, original.replace("\".\"", "\"worktree\"")).unwrap();

    let reference = issue(temp.path(), &["--commit", &head]);
    assert_eq!(reference["commit"], head);
}

#[test]
fn exact_export_contains_only_canonical_graph_families() {
    let temp = committed_store();
    let proposal_dir = temp
        .path()
        .join(".provenance/state/scopes/default/ideation");
    std::fs::create_dir_all(&proposal_dir).unwrap();
    std::fs::write(
        proposal_dir.join("proposal_cards.jsonl"),
        concat!(
            "{\"schema_version\":1,\"scope_id\":\"default\",",
            "\"id\":\"proposal_workflowd_123\",\"proposal_key\":\"workflowd-123\",",
            "\"proposal_type\":\"no_action\",\"title\":\"No graph change\",",
            "\"summary\":\"Collaboration-only proposal\",\"traceability\":{",
            "\"target\":{\"artifact_type\":\"requirement\",\"artifact_id\":\"req_none\"},",
            "\"source_ids\":[],\"evidence_references\":[],\"supporting_claim_ids\":[]},",
            "\"promotion_state\":\"proposed\"}\n"
        ),
    )
    .unwrap();
    git(temp.path(), &["add", ".provenance/state"]);
    git(
        temp.path(),
        &["commit", "-qm", "non-graph collaboration state"],
    );
    let reference = issue(temp.path(), &[]);
    let reference_path = write_reference(temp.path(), &reference);

    let output = provenance(temp.path())
        .args([
            "graph-reference",
            "exact-export",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export: Value = serde_json::from_slice(&output).unwrap();
    let graph = export["graph"].as_object().unwrap();
    assert!(!graph.contains_key("proposal_cards"));
    assert!(!graph.contains_key("promotion_decisions"));
    assert!(!String::from_utf8(output.clone())
        .unwrap()
        .contains("proposal_workflowd_123"));
    assert!(!String::from_utf8(output).unwrap().contains("workflowd-123"));
}

#[test]
fn collaboration_claims_do_not_change_digest_or_appear_in_exact_export() {
    let temp = committed_store();
    provenance(temp.path())
        .args([
            "sources",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "source_origin",
            "--name",
            "Origin metadata source",
            "--origin-thread",
            "thread_private",
            "--origin-message",
            "message_private",
        ])
        .assert()
        .success();
    provenance(temp.path())
        .args([
            "requirements",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "req_claims",
            "--statement",
            "Claims are collaboration metadata",
        ])
        .assert()
        .success();
    provenance(temp.path())
        .args([
            "topics",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "topic_claims",
            "--requirement-id",
            "req_claims",
            "--title",
            "Claim handling",
        ])
        .assert()
        .success();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "add graph topic"]);
    let unclaimed = issue(temp.path(), &[]);

    provenance(temp.path())
        .args([
            "topics",
            "claim",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "topic_claims",
            "--actor",
            "workflowd-123",
        ])
        .assert()
        .success();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "claim graph topic"]);
    let claimed = issue(temp.path(), &[]);

    assert_eq!(unclaimed["graph_digest"], claimed["graph_digest"]);
    let reference_path = write_reference(temp.path(), &claimed);
    let output = provenance(temp.path())
        .args([
            "graph-reference",
            "exact-export",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("claimed_by"));
    assert!(!output.contains("claimed_at"));
    assert!(!output.contains("workflowd-123"));
    assert!(!output.contains("origin_thread"));
    assert!(!output.contains("origin_message"));
    assert!(!output.contains("thread_private"));
    assert!(!output.contains("message_private"));
}

#[test]
fn issue_rejects_unsupported_pinned_record_schema_versions() {
    let temp = committed_store();
    provenance(temp.path())
        .args([
            "sources",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "source_v2",
            "--name",
            "Future source",
        ])
        .assert()
        .success();
    let source_path = temp
        .path()
        .join(".provenance/state/scopes/default/sources/source.jsonl");
    let source = std::fs::read_to_string(&source_path).unwrap();
    std::fs::write(
        &source_path,
        source.replace("\"schema_version\":1", "\"schema_version\":2"),
    )
    .unwrap();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "add unsupported source"]);

    provenance(temp.path())
        .args([
            "graph-reference",
            "issue",
            "--repo",
            ".",
            "--scope",
            "default",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "source 'source_v2' has unsupported schema_version 2",
        ));
}

#[test]
fn issue_rejects_unknown_fields_in_pinned_graph_records() {
    let temp = committed_store();
    provenance(temp.path())
        .args([
            "sources",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "source_typo",
            "--name",
            "Typo source",
        ])
        .assert()
        .success();
    let source_path = temp
        .path()
        .join(".provenance/state/scopes/default/sources/source.jsonl");
    let source = std::fs::read_to_string(&source_path).unwrap();
    std::fs::write(
        &source_path,
        source.replace(
            "\"name\":\"Typo source\"",
            "\"name\":\"Typo source\",\"naem\":\"lost\"",
        ),
    )
    .unwrap();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "add malformed source"]);

    provenance(temp.path())
        .args([
            "graph-reference",
            "issue",
            "--repo",
            ".",
            "--scope",
            "default",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown field"));
}
