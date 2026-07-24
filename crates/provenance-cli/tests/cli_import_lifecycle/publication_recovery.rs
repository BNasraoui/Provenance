use super::support::{init_repo, provenance};

#[test]
fn repository_access_restores_backup_after_interrupted_publication() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    init_repo(&repo, None);
    let transaction = repo.join(".provenance/cache/import-transactions/interrupted");
    std::fs::create_dir_all(&transaction).unwrap();
    std::fs::rename(
        repo.join(".provenance/state"),
        transaction.join("backup-state"),
    )
    .unwrap();
    write_publication_marker(&repo, &transaction, "backup_created");

    check_repo(&repo);

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
    init_repo(&repo, None);
    let transaction = repo.join(".provenance/cache/import-transactions/published");
    std::fs::create_dir_all(transaction.join("backup-state")).unwrap();
    write_publication_marker(&repo, &transaction, "published");

    check_repo(&repo);

    assert!(repo.join(".provenance/state/manifest.json").is_file());
    assert!(!transaction.exists());
    assert!(!repo
        .join(".provenance/cache/import-publication.json")
        .exists());
}

fn check_repo(repo: &std::path::Path) {
    provenance()
        .args([
            "check",
            "--repo",
            repo.to_str().unwrap(),
            "--format",
            "json",
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
