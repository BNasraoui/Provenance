use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;

#[test]
fn check_rejects_dangling_edge_endpoint_in_any_edge_shard() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path());
    let state = dir.path().join(".provenance/state");
    write_jsonl(
        &state.join("scopes/default/requirements/req.jsonl"),
        r#"{"schema_version":1,"scope_id":"default","id":"req_existing","statement":"Existing requirement","status":"active"}"#,
    );
    write_jsonl(
        &state.join("edges/edges-01.jsonl"),
        r#"{"schema_version":1,"scope_id":"default","id":"edge_missing_target","edge_type":"refines_into","from_type":"requirement","from_id":"req_existing","to_type":"requirement","to_id":"req_missing"}"#,
    );

    provenance(dir.path())
        .failure()
        .stderr(contains("dangling reference"))
        .stderr(contains("edge edge_missing_target"))
        .stderr(contains("to requirement req_missing"));
}

#[test]
fn check_rejects_dangling_artifact_links() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path());
    let state = dir.path().join(".provenance/state");
    write_jsonl(
        &state.join("scopes/default/requirements/req.jsonl"),
        r#"{"schema_version":1,"scope_id":"default","id":"req_existing","statement":"Existing requirement","status":"active"}"#,
    );
    write_jsonl(
        &state.join("scopes/default/topics/topic.jsonl"),
        r#"{"schema_version":1,"scope_id":"default","id":"topic_existing","requirement_id":"req_existing","title":"Existing topic","status":"open","links":[{"target_type":"rule","target_id":"rule_missing"}]}"#,
    );

    provenance(dir.path())
        .failure()
        .stderr(contains("dangling reference"))
        .stderr(contains("topic topic_existing"))
        .stderr(contains("link rule rule_missing"));
}

#[test]
fn check_accepts_edges_whose_endpoints_exist_in_different_scopes() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path());
    let state = dir.path().join(".provenance/state");
    std::fs::write(
        state.join("manifest.json"),
        r#"{"schema_version":1,"scopes":[{"id":"frontend","path_prefix":"."},{"id":"platform","path_prefix":"services/platform"}]}"#,
    )
    .unwrap();
    write_jsonl(
        &state.join("scopes/frontend/requirements/req.jsonl"),
        r#"{"schema_version":1,"scope_id":"frontend","id":"req_frontend","statement":"Frontend requirement","status":"active"}"#,
    );
    write_jsonl(
        &state.join("scopes/platform/requirements/req.jsonl"),
        r#"{"schema_version":1,"scope_id":"platform","id":"req_platform","statement":"Platform requirement","status":"active"}"#,
    );
    write_jsonl(
        &state.join("edges/edges-00.jsonl"),
        r#"{"schema_version":1,"scope_id":"frontend","id":"edge_cross_scope","edge_type":"depends_on","from_type":"requirement","from_id":"req_frontend","to_type":"requirement","to_id":"req_platform"}"#,
    );

    provenance(dir.path())
        .success()
        .stdout(contains(r#""status": "ok""#));
}

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

fn provenance(repo: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "check",
            "--repo",
            repo.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
}

fn write_jsonl(path: &Path, record: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, format!("{record}\n")).unwrap();
}
