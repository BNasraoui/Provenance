use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn check_rejects_unsupported_contribution_and_synthesis_versions() {
    for kind in ["contribution", "synthesis"] {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path());
        write_unsupported_record(dir.path(), kind);
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                "check",
                "--repo",
                dir.path().to_str().unwrap(),
                "--format",
                "json",
            ])
            .assert()
            .failure()
            .stderr(contains(format!("{kind} schema_version must be 1")));
    }
}

#[test]
fn import_rejects_unsupported_contribution_and_synthesis_versions() {
    for kind in ["contribution", "synthesis"] {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        init(&repo);
        let export = dir.path().join("export.json");
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                "export",
                "--repo",
                repo.to_str().unwrap(),
                "--scope",
                "default",
                "--format",
                "json",
                "--output",
                export.to_str().unwrap(),
            ])
            .assert()
            .success();
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&export).unwrap()).unwrap();
        if kind == "contribution" {
            value["contributions"] = serde_json::json!([contribution()]);
        } else {
            value["synthesis_packets"] = serde_json::json!([synthesis()]);
        }
        value["requirements"] = serde_json::json!([requirement()]);
        std::fs::write(&export, serde_json::to_vec(&value).unwrap()).unwrap();
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                "import",
                "--repo",
                repo.to_str().unwrap(),
                "--scope",
                "default",
                "--input",
                export.to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(contains(format!("{kind} schema_version must be 1")));
    }
}

fn init(repo: &std::path::Path) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
        ])
        .assert()
        .success();
}

fn write_unsupported_record(repo: &std::path::Path, kind: &str) {
    let requirements = repo.join(".provenance/state/scopes/default/requirements");
    std::fs::create_dir_all(&requirements).unwrap();
    std::fs::write(
        requirements.join("req.jsonl"),
        format!("{}\n", serde_json::to_string(&requirement()).unwrap()),
    )
    .unwrap();
    let ideation = repo.join(".provenance/state/scopes/default/ideation");
    std::fs::create_dir_all(&ideation).unwrap();
    let (name, value) = if kind == "contribution" {
        ("contributions.jsonl", contribution())
    } else {
        ("synthesis_packets.jsonl", synthesis())
    };
    std::fs::write(
        ideation.join(name),
        format!("{}\n", serde_json::to_string(&value).unwrap()),
    )
    .unwrap();
}

fn requirement() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "req_future",
        "statement": "Future target", "status": "active"
    })
}

fn contribution() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 2, "scope_id": "default", "id": "contribution_future",
        "target": {"artifact_type": "requirement", "artifact_id": "req_future"},
        "participant_slot": "future", "stance": "support", "strongest_finding": "Future",
        "evidence_references": [], "material_claims": [], "risks": [], "objections": [],
        "challenges": [], "suggested_artifact_changes": [], "unsupported_recommendations": [],
        "uncertainty": {"level": "low", "rationale": "Future"}, "open_questions": []
    })
}

fn synthesis() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 2, "scope_id": "default", "id": "synthesis_future",
        "target": {"artifact_type": "requirement", "artifact_id": "req_future"},
        "summary": "Future", "consensus": [], "contested_claims": [], "minority_objections": [],
        "evidence_gaps": [], "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [], "required_human_decisions": []
    })
}
