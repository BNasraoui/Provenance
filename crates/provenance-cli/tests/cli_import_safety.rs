use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::path::Path;

fn init(repo: &Path) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();
}

fn empty_export() -> Value {
    json!({
        "scope": "default",
        "sources": [],
        "requirements": [],
        "resolutions": [],
        "rules": [],
        "edges": [],
        "threads": [],
        "messages": []
    })
}

fn import(repo: &Path, input: &Path, dry_run: bool) -> assert_cmd::assert::Assert {
    let mut command = Command::cargo_bin("provenance").unwrap();
    command.args([
        "import",
        "--repo",
        repo.to_str().unwrap(),
        "--scope",
        "default",
        "--input",
        input.to_str().unwrap(),
        "--format",
        "json",
    ]);
    if dry_run {
        command.arg("--dry-run");
    }
    command.assert()
}

#[test]
fn import_rejects_foreign_embedded_scope_in_dry_run_and_write_modes() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    let input = dir.path().join("foreign.json");
    let mut export = empty_export();
    export["sources"] = json!([{
        "schema_version": 1,
        "scope_id": "foreign",
        "id": "source_foreign",
        "name": "foreign",
        "source_type": "system_state"
    }]);
    std::fs::write(&input, serde_json::to_vec(&export).unwrap()).unwrap();

    for dry_run in [true, false] {
        import(&repo, &input, dry_run)
            .failure()
            .stderr(predicate::str::contains(
                "source source_foreign belongs to scope foreign",
            ));
    }

    export["sources"] = json!([]);
    export["edges"] = json!([{
        "schema_version":1,"scope_id":"foreign","id":"edge_foreign",
        "edge_type":"produces","from_type":"requirement","from_id":"req_foreign",
        "to_type":"rule","to_id":"rule_foreign"
    }]);
    std::fs::write(&input, serde_json::to_vec(&export).unwrap()).unwrap();
    for dry_run in [true, false] {
        import(&repo, &input, dry_run)
            .failure()
            .stderr(predicate::str::contains(
                "edge edge_foreign belongs to scope foreign",
            ));
    }
}

#[test]
fn import_rejects_duplicate_source_ids_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    let input = dir.path().join("duplicates.json");
    let mut export = empty_export();
    export["sources"] = json!([
        {"schema_version":1,"scope_id":"default","id":"source_dup","name":"one","source_type":"system_state"},
        {"schema_version":1,"scope_id":"default","id":"source_dup","name":"two","source_type":"system_state"}
    ]);
    std::fs::write(&input, serde_json::to_vec(&export).unwrap()).unwrap();

    import(&repo, &input, true)
        .failure()
        .stderr(predicate::str::contains("duplicate source id source_dup"));
}

#[test]
fn import_preserves_other_scopes_global_edges() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    let edges = repo.join(".provenance/state/edges/edges-00.jsonl");
    std::fs::create_dir_all(edges.parent().unwrap()).unwrap();
    let foreign = r#"{"schema_version":1,"scope_id":"foreign","id":"edge_foreign","edge_type":"produces","from_type":"requirement","from_id":"req_foreign","to_type":"rule","to_id":"rule_foreign"}"#;
    std::fs::write(&edges, format!("{foreign}\n")).unwrap();
    let input = dir.path().join("default.json");
    std::fs::write(&input, serde_json::to_vec(&empty_export()).unwrap()).unwrap();

    import(&repo, &input, false).success();

    assert_eq!(
        std::fs::read_to_string(edges).unwrap(),
        format!("{foreign}\n")
    );
}

#[test]
fn import_rejects_zero_evidence_lines() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    let input = dir.path().join("line-zero.json");
    let mut export = empty_export();
    export["proposal_cards"] = json!([{
        "schema_version": 1,
        "scope_id": "default",
        "id": "proposal_zero",
        "proposal_key": "review/zero",
        "proposal_type": "requirement_candidate",
        "title": "zero",
        "summary": "zero",
        "traceability": {
            "target": {"artifact_type":"source","artifact_id":"source_code"},
            "source_ids": ["source_code"],
            "evidence_references": [{
                "reference_id":"evidence_zero",
                "evidence_type":"artifact",
                "summary":"bad line",
                "file_path":"src/lib.rs",
                "line":0
            }],
            "supporting_claim_ids": []
        },
        "promotion_state": "proposed"
    }]);
    std::fs::write(&input, serde_json::to_vec(&export).unwrap()).unwrap();

    import(&repo, &input, true)
        .failure()
        .stderr(predicate::str::contains("evidence line must be at least 1"));
}

#[test]
fn import_io_failure_leaves_the_old_generation_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    let sources = repo.join(".provenance/state/scopes/default/sources/source.jsonl");
    std::fs::create_dir_all(sources.parent().unwrap()).unwrap();
    let old = r#"{"schema_version":1,"scope_id":"default","id":"source_old","name":"old","source_type":"system_state"}"#;
    std::fs::write(&sources, format!("{old}\n")).unwrap();

    let blocked_parent = repo.join(".provenance/state/scopes/default/topics");
    if blocked_parent.exists() {
        std::fs::remove_dir_all(&blocked_parent).unwrap();
    }
    std::fs::write(&blocked_parent, "injected I/O obstruction").unwrap();
    let input = dir.path().join("replacement.json");
    let mut export = empty_export();
    export["sources"] = json!([{
        "schema_version":1,"scope_id":"default","id":"source_new",
        "name":"new","source_type":"system_state"
    }]);
    std::fs::write(&input, serde_json::to_vec(&export).unwrap()).unwrap();

    import(&repo, &input, false).failure();

    assert_eq!(
        std::fs::read_to_string(sources).unwrap(),
        format!("{old}\n")
    );
}
