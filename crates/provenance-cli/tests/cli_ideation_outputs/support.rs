use assert_cmd::Command;

pub(super) fn init_repo(repo: &str) {
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

pub(super) fn create_source(repo: &str, id: &str) {
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
            id,
            "--name",
            "Codebase",
            "--source-type",
            "system_state",
            "--format",
            "json",
        ])
        .assert()
        .success();
}
