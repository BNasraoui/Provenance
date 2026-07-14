use assert_cmd::Command;

#[test]
fn import_rejects_forged_lifecycle_state_and_foreign_scope_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    init_and_seed(&repo);
    let baseline = export_value(&repo, dir.path().join("baseline.json"));

    for state in [
        "asserted",
        "swarm_asserted",
        "accepted",
        "rejected",
        "deferred",
    ] {
        let mut candidate = baseline.clone();
        candidate["proposal_cards"] = serde_json::json!([proposal(state)]);
        let input = dir.path().join(format!("forged-{state}.json"));
        std::fs::write(&input, serde_json::to_vec(&candidate).unwrap()).unwrap();
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                "import",
                "--repo",
                &repo,
                "--scope",
                "default",
                "--input",
                input.to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(predicates::str::contains("proposals must begin proposed"));
    }

    let mut foreign = baseline;
    foreign["sources"][0]["scope_id"] = serde_json::json!("foreign");
    let input = dir.path().join("foreign.json");
    std::fs::write(&input, serde_json::to_vec(&foreign).unwrap()).unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--input",
            input.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "source scope_id must match imported scope",
        ));
}

#[test]
fn late_validation_failure_leaves_every_live_shard_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    init_and_seed(&repo);
    let before = export_value(&repo, dir.path().join("before.json"));
    let mut candidate = before.clone();
    for field in [
        "sources",
        "domains",
        "requirements",
        "boundaries",
        "topics",
        "questions",
        "resolutions",
        "rules",
        "services",
        "service_bindings",
        "edges",
        "threads",
        "messages",
        "contributions",
        "synthesis_packets",
        "proposal_cards",
        "assertion_records",
        "promotion_decisions",
    ] {
        candidate[field] = serde_json::json!([]);
    }
    candidate["proposal_cards"] = serde_json::json!([proposal("proposed")]);
    candidate["edges"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "edge_late_failure",
        "edge_type": "depends_on",
        "from_type": "requirement", "from_id": "req_anchor",
        "to_type": "requirement", "to_id": "req_missing"
    }]);
    let input = dir.path().join("late-invalid.json");
    std::fs::write(&input, serde_json::to_vec(&candidate).unwrap()).unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--input",
            input.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("req_missing"));

    let after = export_value(&repo, dir.path().join("after.json"));
    assert_eq!(after, before);
}

fn proposal(state: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "proposal_imported",
        "proposal_key": "imported", "proposal_type": "requirement_candidate",
        "title": "Imported", "summary": "Imported candidate",
        "traceability": {
            "target": {"artifact_type": "requirement", "artifact_id": "req_anchor"},
            "source_ids": [], "evidence_references": [], "supporting_claim_ids": []
        },
        "promotion_state": state
    })
}

fn init_and_seed(repo: &str) {
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
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "sources",
            "create",
            "--repo",
            repo,
            "--scope",
            "default",
            "--id",
            "source_anchor",
            "--name",
            "Anchor",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "requirements",
            "create",
            "--repo",
            repo,
            "--scope",
            "default",
            "--id",
            "req_anchor",
            "--statement",
            "Anchor requirement",
        ])
        .assert()
        .success();
}

fn export_value(repo: &str, path: std::path::PathBuf) -> serde_json::Value {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export",
            "--repo",
            repo,
            "--scope",
            "default",
            "--format",
            "json",
            "--output",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
