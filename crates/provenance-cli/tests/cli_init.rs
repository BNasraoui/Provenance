use assert_cmd::Command;
use serde_json::Value;

fn init(repo: &std::path::Path, args: &[&str]) -> assert_cmd::assert::Assert {
    let mut command = Command::cargo_bin("provenance").unwrap();
    command.args(["init", "--path", repo.to_str().unwrap()]);
    command.args(args).assert()
}

fn read_manifest(repo: &std::path::Path) -> Value {
    serde_json::from_slice(&std::fs::read(repo.join(".provenance/state/manifest.json")).unwrap())
        .unwrap()
}

fn write_manifest(repo: &std::path::Path, manifest: &Value) {
    std::fs::write(
        repo.join(".provenance/state/manifest.json"),
        serde_json::to_vec_pretty(manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn cli_init_check_and_materialize_empty_repo() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");

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

    assert!(repo.join(".provenance/state/manifest.json").exists());
    assert!(repo.join(".provenance/cache").exists());
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(repo.join(".provenance/state/manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["disposition_actor_ids"], serde_json::json!([]));

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

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "materialize",
            "--repo",
            repo.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success();

    assert!(repo.join(".provenance/cache/provenance.db").exists());
}

#[test]
fn fresh_init_without_scope_fails_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");

    init(&repo, &[]).failure();

    assert!(!repo.exists());
}

#[test]
fn init_rerun_without_manifest_flags_preserves_every_manifest_field() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init(
        &repo,
        &[
            "--scope",
            "default",
            "--path-prefix",
            "src",
            "--disposition-actor-id",
            "reviewer",
        ],
    )
    .success();
    let mut original = read_manifest(&repo);
    original["scopes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"id": "docs", "path_prefix": "docs"}));
    write_manifest(&repo, &original);

    init(&repo, &[]).success();

    assert_eq!(read_manifest(&repo), original);
}

/// A future manifest is refused outright, not preserved: the guard-all-reads
/// ruling covers init like every other read, so re-init never rewrites a
/// manifest this build cannot understand.
#[test]
fn init_rerun_refuses_a_future_manifest_version() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init(&repo, &["--scope", "default"]).success();
    let mut original = read_manifest(&repo);
    original["schema_version"] = serde_json::json!(2);
    write_manifest(&repo, &original);

    init(&repo, &[])
        .failure()
        .stderr(predicates::str::contains("schema_version must be 1"));

    assert_eq!(read_manifest(&repo), original);
}

#[test]
fn init_rerun_with_actor_flag_updates_only_the_allowlist() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init(
        &repo,
        &[
            "--scope",
            "default",
            "--path-prefix",
            "src",
            "--disposition-actor-id",
            "reviewer",
        ],
    )
    .success();
    let original = read_manifest(&repo);

    init(
        &repo,
        &[
            "--disposition-actor-id",
            "maintainer",
            "--disposition-actor-id",
            "release-manager",
        ],
    )
    .success();

    let mut expected = original;
    expected["disposition_actor_ids"] = serde_json::json!(["maintainer", "release-manager"]);
    assert_eq!(read_manifest(&repo), expected);
}

#[test]
fn init_clear_disposition_actors_only_empties_the_allowlist() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init(
        &repo,
        &[
            "--scope",
            "default",
            "--path-prefix",
            "src",
            "--disposition-actor-id",
            "reviewer",
        ],
    )
    .success();
    let original = read_manifest(&repo);

    init(&repo, &["--clear-disposition-actors"]).success();

    let mut expected = original;
    expected["disposition_actor_ids"] = serde_json::json!([]);
    assert_eq!(read_manifest(&repo), expected);
}
