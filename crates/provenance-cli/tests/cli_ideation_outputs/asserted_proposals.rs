use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

fn proposal(repo: &str, id: &str, state: &str, builds_on: Option<&str>) {
    let mut command = Command::cargo_bin("provenance").unwrap();
    command.args([
        "proposals",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        id,
        "--proposal-key",
        id,
        "--proposal-type",
        "requirement_candidate",
        "--title",
        id,
        "--summary",
        "Observed behavior",
        "--target-type",
        "requirement",
        "--target-id",
        "req_anchor",
        "--promotion-state",
        state,
        "--format",
        "json",
    ]);
    if let Some(parent) = builds_on {
        command.args(["--builds-on", parent]);
    }
    command.assert().success();
}

#[test]
fn asserted_proposals_are_consultable_and_can_be_built_on_provisionally() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            &repo,
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();

    proposal(&repo, "proposal_asserted", "asserted", None);
    proposal(
        &repo,
        "proposal_derivative",
        "proposed",
        Some("proposal_asserted"),
    );

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "list",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--promotion-state",
            "asserted",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("proposal_asserted"))
        .stdout(predicates::str::contains("proposal_derivative").not());

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "prime", "--repo", &repo, "--scope", "default", "--format", "markdown",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "proposal_asserted [asserted; not human-ratified]",
        ))
        .stdout(predicates::str::contains(
            "builds on provisionally: proposal_asserted",
        ));
}
