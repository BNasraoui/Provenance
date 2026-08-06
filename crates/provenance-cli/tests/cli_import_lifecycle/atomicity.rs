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

#[cfg(unix)]
#[test]
fn import_rejects_external_file_symlink_without_changing_live_state() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo, None);
    let external = dir.path().join("secret");
    std::fs::write(&external, "do not import").unwrap();
    let link = repo.join(".provenance/state/external-link");
    std::os::unix::fs::symlink(&external, &link).unwrap();
    let input = dir.path().join("input.json");
    export_scope(&repo, &input).success();

    import_scope(&repo, &input)
        .failure()
        .stderr(predicates::str::contains("unsupported state entry"));

    let metadata = std::fs::symlink_metadata(&link).unwrap();
    assert!(metadata.file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(&external).unwrap(), "do not import");
}

#[cfg(unix)]
#[test]
fn dry_run_rejects_symlinked_import_transactions_directory() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo, None);
    let input = dir.path().join("input.json");
    export_scope(&repo, &input).success();
    let outside = dir.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    let transactions = repo.join(".provenance/cache/import-transactions");
    if transactions.exists() {
        std::fs::remove_dir_all(&transactions).unwrap();
    }
    std::os::unix::fs::symlink(&outside, &transactions).unwrap();

    super::support::provenance()
        .args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            input.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("repository cache"));

    let outside_entries = std::fs::read_dir(outside)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert!(
        outside_entries.is_empty(),
        "outside writes: {outside_entries:?}"
    );
}

#[cfg(unix)]
#[test]
fn dry_run_rejects_symlinked_cache_before_locking() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo, None);
    let input = dir.path().join("input.json");
    export_scope(&repo, &input).success();
    let outside = dir.path().join("outside-cache");
    std::fs::create_dir(&outside).unwrap();
    let cache = repo.join(".provenance/cache");
    std::fs::remove_dir_all(&cache).unwrap();
    std::os::unix::fs::symlink(&outside, &cache).unwrap();

    super::support::provenance()
        .args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            input.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("symlink component"));

    let outside_entries = std::fs::read_dir(outside)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert!(
        outside_entries.is_empty(),
        "outside writes: {outside_entries:?}"
    );
}
