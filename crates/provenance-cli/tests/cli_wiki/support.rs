use assert_cmd::Command;

pub fn init_repo(repo: &str) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo,
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();
}

pub fn seed_full_state(dir: &std::path::Path, repo: &str) {
    init_repo(repo);
    let import_path = dir.join("state.json");
    std::fs::write(&import_path, STATE).unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            repo,
            "--scope",
            "default",
            "--input",
            &import_path.to_string_lossy(),
            "--format",
            "json",
        ])
        .assert()
        .success();
    create_edge(
        repo,
        "resolves",
        "resolution",
        "res_sah",
        "requirement",
        "req_sah",
    );
    create_edge(
        repo,
        "produces",
        "resolution",
        "res_sah",
        "rule",
        "rule_sah_001",
    );
}

fn create_edge(repo: &str, edge: &str, from_type: &str, from: &str, to_type: &str, to: &str) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "edges",
            "create",
            "--repo",
            repo,
            "--scope",
            "default",
            "--type",
            edge,
            "--from-type",
            from_type,
            "--from-id",
            from,
            "--to-type",
            to_type,
            "--to-id",
            to,
        ])
        .assert()
        .success();
}

const STATE: &str = r#"{
  "scope": "default",
  "sources": [{"schema_version":1,"scope_id":"default","id":"source_sah","name":"Support at Home","source_type":"legislation","url":"https://example.test/sah","reference":"Department guidance"}],
  "domains": [{"schema_version":1,"scope_id":"default","id":"domain_care","name":"Care delivery","description":"Requirements governing in-home care"}],
  "requirements": [
    {"schema_version":1,"scope_id":"default","id":"req_gap","statement":"Uncited requirement","status":"active","source_refs":[]},
    {"schema_version":1,"scope_id":"default","id":"req_sah","statement":"Support at Home shall be traceable","status":"active","domain_id":"domain_care","source_refs":[{"source_id":"source_sah","clause":"Program overview"}]}
  ],
  "resolutions": [{"schema_version":1,"scope_id":"default","id":"res_sah","title":"SAH extraction","position":"Keep as draft extraction","rationale":"Needs human review","status":"approved","review_on":null,"review_triggers":[]}],
  "rules": [{"schema_version":1,"scope_id":"default","id":"rule_sah_001","rule_code":"SAH-001","name":"SAH rule","statement":"Draft rule shall stay draft","status":"active","severity":"high","rule_type":"business","modality":"obligation","source_document":"Example-API-main/src/example.php","source_section":"lines 1-3","expression":{},"inputs":[]}],
  "edges": [], "threads": [], "messages": []
}"#;
