use assert_cmd::Command;

#[test]
fn forged_terminal_import_fails_without_changing_live_scope() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
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
    let baseline = dir.path().join("baseline.json");
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
            baseline.to_str().unwrap(),
        ])
        .assert()
        .success();
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
    std::fs::write(&input, serde_json::to_vec(&forged).unwrap()).unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            input.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("frozen shipped-v1 fingerprint"));

    let after = dir.path().join("after.json");
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
            after.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(std::fs::read(after).unwrap(), before);
}

#[test]
fn late_scope_validation_failure_is_atomic() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
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
    let baseline = dir.path().join("baseline.json");
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
            baseline.to_str().unwrap(),
        ])
        .assert()
        .success();
    let before = std::fs::read(&baseline).unwrap();
    let mut invalid: serde_json::Value = serde_json::from_slice(&before).unwrap();
    invalid["edges"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "edge_invalid",
        "edge_type": "depends_on", "from_type": "requirement", "from_id": "req_missing_a",
        "to_type": "requirement", "to_id": "req_missing_b"
    }]);
    let input = dir.path().join("invalid.json");
    std::fs::write(&input, serde_json::to_vec(&invalid).unwrap()).unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            input.to_str().unwrap(),
        ])
        .assert()
        .failure();

    let after = dir.path().join("after.json");
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
            after.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(std::fs::read(after).unwrap(), before);
}

#[test]
fn repository_access_restores_backup_after_interrupted_publication() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo);
    let transaction = repo.join(".provenance/cache/import-transactions/interrupted");
    std::fs::create_dir_all(&transaction).unwrap();
    std::fs::rename(
        repo.join(".provenance/state"),
        transaction.join("backup-state"),
    )
    .unwrap();
    write_publication_marker(&repo, &transaction, "backup_created");

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
        .success();

    assert!(repo.join(".provenance/state/manifest.json").is_file());
    assert!(!repo
        .join(".provenance/cache/import-publication.json")
        .exists());
    assert!(!transaction.exists());
}

#[test]
fn repository_access_finishes_cleanup_after_published_state() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo);
    let transaction = repo.join(".provenance/cache/import-transactions/published");
    std::fs::create_dir_all(transaction.join("backup-state")).unwrap();
    write_publication_marker(&repo, &transaction, "published");

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
        .success();

    assert!(repo.join(".provenance/state/manifest.json").is_file());
    assert!(!transaction.exists());
    assert!(!repo
        .join(".provenance/cache/import-publication.json")
        .exists());
}

fn init_repo(repo: &std::path::Path) {
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

fn write_publication_marker(repo: &std::path::Path, transaction: &std::path::Path, phase: &str) {
    std::fs::write(
        repo.join(".provenance/cache/import-publication.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "transaction_dir": transaction,
            "phase": phase
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn shipped_legacy_export_imports_into_fresh_repo_and_materializes() {
    let dir = tempfile::tempdir().unwrap();
    let fresh = dir.path().join("fresh");
    let export = dir.path().join("shipped.json");
    let shipped = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export",
            "--repo",
            shipped.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
            "--output",
            export.to_str().unwrap(),
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            fresh.to_str().unwrap(),
            "--scope",
            "default",
            "--disposition-actor-id",
            "codex-review-panel-gpt55-medium",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            fresh.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            export.to_str().unwrap(),
        ])
        .assert()
        .success();
    for command in ["check", "materialize"] {
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                command,
                "--repo",
                fresh.to_str().unwrap(),
                "--format",
                "json",
            ])
            .assert()
            .success();
    }
}

#[test]
fn historical_shipped_manifest_without_actor_allowlist_remains_readable() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("historical");
    let shipped = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(".provenance/state");
    copy_tree(&shipped, &repo.join(".provenance/state"));
    let manifest_path = repo.join(".provenance/state/manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest
        .as_object_mut()
        .unwrap()
        .remove("disposition_actor_ids");
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    for command in ["check", "materialize"] {
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                command,
                "--repo",
                repo.to_str().unwrap(),
                "--format",
                "json",
            ])
            .assert()
            .success();
    }
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
        ])
        .assert()
        .success();
}

fn copy_tree(source: &std::path::Path, destination: &std::path::Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn one_byte_change_to_shipped_legacy_terminal_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let fresh = dir.path().join("fresh");
    let export = dir.path().join("forged-shipped.json");
    let shipped = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export",
            "--repo",
            shipped.to_str().unwrap(),
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
    let terminal = value["proposal_cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|proposal| proposal["promotion_state"] != "proposed")
        .unwrap();
    terminal["summary"] = serde_json::json!(format!("{}x", terminal["summary"].as_str().unwrap()));
    std::fs::write(&export, serde_json::to_vec(&value).unwrap()).unwrap();
    init_repo(&fresh);
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            fresh.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            export.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("frozen shipped-v1 fingerprint"));
}
