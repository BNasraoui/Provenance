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
            "--human-authority-id",
            "ben",
        ])
        .assert()
        .success();
}

pub fn create_source(repo: &str, id: &str) {
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
