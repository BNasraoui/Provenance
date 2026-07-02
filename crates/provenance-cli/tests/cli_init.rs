use assert_cmd::Command;

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
