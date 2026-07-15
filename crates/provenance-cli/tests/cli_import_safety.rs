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

fn sample_records() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "sources",
            "source",
            json!({"schema_version":1,"scope_id":"default","id":"source_dup","name":"source","source_type":"system_state"}),
        ),
        (
            "domains",
            "domain",
            json!({"schema_version":1,"scope_id":"default","id":"domain_dup","name":"domain"}),
        ),
        (
            "requirements",
            "requirement",
            json!({"schema_version":1,"scope_id":"default","id":"req_dup","statement":"requirement","status":"discovery"}),
        ),
        (
            "boundaries",
            "boundary",
            json!({"schema_version":1,"scope_id":"default","id":"boundary_dup","requirement_id":"req_dup","statement":"boundary"}),
        ),
        (
            "topics",
            "topic",
            json!({"schema_version":1,"scope_id":"default","id":"topic_dup","requirement_id":"req_dup","title":"topic","status":"open"}),
        ),
        (
            "questions",
            "question",
            json!({"schema_version":1,"scope_id":"default","id":"question_dup","topic_id":"topic_dup","requirement_id":"req_dup","question":"question?","resolution_method":"verify","status":"open"}),
        ),
        (
            "resolutions",
            "resolution",
            json!({"schema_version":1,"scope_id":"default","id":"resolution_dup","title":"resolution","position":"position","rationale":"rationale","status":"draft","review_on":null}),
        ),
        (
            "rules",
            "rule",
            json!({"schema_version":1,"scope_id":"default","id":"rule_dup","rule_code":"DUP","statement":"rule","status":"draft","severity":"medium"}),
        ),
        (
            "services",
            "service",
            json!({"schema_version":1,"scope_id":"default","id":"service_dup","name":"service","status":"active"}),
        ),
        (
            "service_bindings",
            "service binding",
            json!({"schema_version":1,"scope_id":"default","id":"binding_dup","rule_id":"rule_dup","service_id":"service_dup","binding_type":"enforces"}),
        ),
        (
            "edges",
            "edge",
            json!({"schema_version":1,"scope_id":"default","id":"edge_dup","edge_type":"produces","from_type":"requirement","from_id":"req_dup","to_type":"rule","to_id":"rule_dup"}),
        ),
        (
            "threads",
            "thread",
            json!({"schema_version":1,"scope_id":"default","id":"thread_dup","parent":{"node_type":"requirement","node_id":"req_dup"},"status":"active","created_at":1}),
        ),
        (
            "messages",
            "message",
            json!({"schema_version":1,"scope_id":"default","id":"message_dup","thread_id":"thread_dup","role":"user","body":"message","created_at":1}),
        ),
        (
            "contributions",
            "contribution",
            json!({"schema_version":1,"scope_id":"default","id":"contribution_dup","target":{"artifact_type":"requirement","artifact_id":"req_dup"},"participant_slot":"reviewer","stance":"support","strongest_finding":"finding","evidence_references":[],"material_claims":[],"risks":[],"objections":[],"challenges":[],"suggested_artifact_changes":[],"unsupported_recommendations":[],"uncertainty":{"level":"low","rationale":"direct"},"open_questions":[]}),
        ),
        (
            "synthesis_packets",
            "synthesis packet",
            json!({"schema_version":1,"scope_id":"default","id":"synthesis_dup","target":{"artifact_type":"requirement","artifact_id":"req_dup"},"summary":"summary","consensus":[],"contested_claims":[],"minority_objections":[],"evidence_gaps":[],"unsupported_speculation":[],"open_questions":[],"suggested_artifacts":[],"required_human_decisions":[]}),
        ),
        (
            "proposal_cards",
            "proposal",
            json!({"schema_version":1,"scope_id":"default","id":"proposal_dup","proposal_key":"review/dup","proposal_type":"requirement_candidate","title":"proposal","summary":"summary","traceability":{"target":{"artifact_type":"requirement","artifact_id":"req_dup"},"source_ids":[],"evidence_references":[],"supporting_claim_ids":[]},"promotion_state":"proposed"}),
        ),
        (
            "promotion_decisions",
            "promotion decision",
            json!({"schema_version":1,"scope_id":"default","id":"decision_dup","proposal_id":"proposal_dup","decision":"accepted","rationale":"rationale","actor":{"identity_type":"human","id":"reviewer"}}),
        ),
    ]
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
fn import_rejects_scope_absent_from_manifest_in_dry_run_and_write_modes() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    let input = dir.path().join("absent.json");
    let mut export = empty_export();
    export["scope"] = json!("absent");
    std::fs::write(&input, serde_json::to_vec(&export).unwrap()).unwrap();

    for dry_run in [true, false] {
        let mut command = Command::cargo_bin("provenance").unwrap();
        command.args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "absent",
            "--input",
            input.to_str().unwrap(),
        ]);
        if dry_run {
            command.arg("--dry-run");
        }
        command
            .assert()
            .failure()
            .stderr(predicate::str::contains("absent from manifest"));
        assert!(!repo.join(".provenance/state/scopes/absent").exists());
    }
}

#[test]
fn import_rejects_duplicate_ids_in_every_collection_without_writes() {
    for (collection, kind, record) in sample_records() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        init(&repo);
        let sources = repo.join(".provenance/state/scopes/default/sources/source.jsonl");
        let old = r#"{"schema_version":1,"scope_id":"default","id":"source_old","name":"old","source_type":"system_state"}"#;
        std::fs::create_dir_all(sources.parent().unwrap()).unwrap();
        std::fs::write(&sources, format!("{old}\n")).unwrap();
        let input = dir.path().join("duplicates.json");
        let mut export = empty_export();
        export[collection] = Value::Array(vec![record.clone(), record]);
        std::fs::write(&input, serde_json::to_vec(&export).unwrap()).unwrap();

        import(&repo, &input, false)
            .failure()
            .stderr(predicate::str::contains(format!("duplicate {kind} id")));
        assert_eq!(
            std::fs::read_to_string(&sources).unwrap(),
            format!("{old}\n"),
            "duplicate {kind} import wrote state"
        );
    }
}

#[test]
fn import_removes_selected_scope_from_every_global_edge_shard_and_preserves_foreign_scopes() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    let edges_dir = repo.join(".provenance/state/edges");
    let first = edges_dir.join("edges-00.jsonl");
    let second = edges_dir.join("edges-01.jsonl");
    std::fs::create_dir_all(&edges_dir).unwrap();
    let foreign = r#"{"schema_version":1,"scope_id":"foreign","id":"edge_foreign","edge_type":"produces","from_type":"requirement","from_id":"req_foreign","to_type":"rule","to_id":"rule_foreign"}"#;
    let selected = r#"{"schema_version":1,"scope_id":"default","id":"edge_selected","edge_type":"produces","from_type":"requirement","from_id":"req_selected","to_type":"rule","to_id":"rule_selected"}"#;
    std::fs::write(&first, format!("{selected}\n{foreign}\n")).unwrap();
    std::fs::write(&second, format!("{foreign}\n{selected}\n")).unwrap();
    let input = dir.path().join("default.json");
    std::fs::write(&input, serde_json::to_vec(&empty_export()).unwrap()).unwrap();

    import(&repo, &input, false).success();

    assert_eq!(
        std::fs::read_to_string(first).unwrap(),
        format!("{foreign}\n")
    );
    assert_eq!(
        std::fs::read_to_string(second).unwrap(),
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
