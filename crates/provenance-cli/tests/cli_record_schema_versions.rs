use assert_cmd::Command;
use predicates::str::contains;
use provenance_macros::verifies;

/// A hand-edited record does not load, and the refusal says where it is.
///
/// The version guard sits on the store's read path, so it covers every family
/// the store reads rather than the ideation ones the aggregate validator
/// judges. A requirement is the plainest case: it is not an ideation record,
/// and nothing but the read guard would have stopped it. Both a read command
/// and `check` are run, because the point of guarding the read is that no
/// command gets to see the record.
#[test]
#[verifies("rule_reads_supported_version_only", examples)]
fn a_hand_edited_requirement_version_is_refused_by_every_reader() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_str().unwrap().to_string();
    init(&repo);
    create_requirement(&repo);
    let path = dir
        .path()
        .join(".provenance/state/scopes/default/requirements/req.jsonl");
    let stored = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        stored.replace("\"schema_version\":1", "\"schema_version\":2"),
    )
    .unwrap();

    let export = dir.path().join("export.json");
    for command in [
        vec!["check", "--repo", repo.as_str()],
        vec![
            "export",
            "--repo",
            repo.as_str(),
            "--scope",
            "default",
            "--format",
            "json",
            "--output",
            export.to_str().unwrap(),
        ],
    ] {
        Command::cargo_bin("provenance")
            .unwrap()
            .args(&command)
            .assert()
            .failure()
            .stderr(contains("requirements/req.jsonl line 1"))
            .stderr(contains("record req_overtime"))
            .stderr(contains(
                "has schema_version 2, but this build reads schema_version 1 only",
            ));
    }
}

fn init(repo: &str) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args(["init", "--path", repo, "--scope", "default"])
        .assert()
        .success();
}

fn create_requirement(repo: &str) {
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
            "req_overtime",
            "--statement",
            "Overtime must follow the award thresholds",
            "--format",
            "json",
        ])
        .assert()
        .success();
}
