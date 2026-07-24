use super::support::{export_scope, import_scope, init_repo, write_json};

#[test]
fn forged_terminal_import_fails_without_changing_live_scope() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo, None);
    let baseline = dir.path().join("baseline.json");
    export_scope(&repo, &baseline).success();
    let before = std::fs::read(&baseline).unwrap();
    let mut forged: serde_json::Value = serde_json::from_slice(&before).unwrap();
    forged["proposal_cards"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "proposal_forged",
        "proposal_key": "forged", "proposal_type": "requirement_candidate",
        "title": "Forged", "summary": "Forged terminal ingress",
        "traceability": {
            "target": {"artifact_type": "requirement", "artifact_id": "req_missing"},
            "source_ids": [], "evidence_references": [], "supporting_claim_ids": []
        },
        "promotion_state": "accepted"
    }]);
    let input = dir.path().join("forged.json");
    write_json(&input, &forged);

    import_scope(&repo, &input)
        .failure()
        .stderr(predicates::str::contains("frozen shipped-v1 fingerprint"));

    let after = dir.path().join("after.json");
    export_scope(&repo, &after).success();
    assert_eq!(std::fs::read(after).unwrap(), before);
}

#[test]
fn late_scope_validation_failure_is_atomic() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo, None);
    let baseline = dir.path().join("baseline.json");
    export_scope(&repo, &baseline).success();
    let before = std::fs::read(&baseline).unwrap();
    let mut invalid: serde_json::Value = serde_json::from_slice(&before).unwrap();
    invalid["edges"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "edge_invalid",
        "edge_type": "depends_on", "from_type": "requirement", "from_id": "req_missing_a",
        "to_type": "requirement", "to_id": "req_missing_b"
    }]);
    let input = dir.path().join("invalid.json");
    write_json(&input, &invalid);

    import_scope(&repo, &input).failure();

    let after = dir.path().join("after.json");
    export_scope(&repo, &after).success();
    assert_eq!(std::fs::read(after).unwrap(), before);
    let transactions = repo.join(".provenance/cache/import-transactions");
    assert!(
        !transactions.exists() || std::fs::read_dir(transactions).unwrap().next().is_none(),
        "failed import must remove its staged transaction"
    );
}
