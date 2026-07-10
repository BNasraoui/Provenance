use super::support::{
    create_source, init_repo, write_bad_nested_stable_id, write_bad_proposal_confidence,
    write_merge_output_with_unknown_key,
};
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn swarm_backtrace_land_rejects_bad_proposal_confidence_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let run_dir = dir.path().join("run");
    init_repo(&repo);
    create_source(&repo);
    write_bad_proposal_confidence(&run_dir);
    let run_dir = run_dir.to_string_lossy().to_string();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "swarm-backtrace",
            "land",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--run-dir",
            &run_dir,
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "confidence must be between 0.0 and 1.0",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("contrib_backtrace_extract_auth").not())
        .stdout(predicates::str::contains("contrib_backtrace_refute_auth").not())
        .stdout(predicates::str::contains("synth_backtrace_auth").not())
        .stdout(predicates::str::contains("prop_req_publish_requires_worker").not());
}

#[test]
fn swarm_backtrace_land_rejects_unknown_merge_output_keys() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let run_dir = dir.path().join("run");
    init_repo(&repo);
    create_source(&repo);
    write_merge_output_with_unknown_key(&run_dir);
    let run_dir = run_dir.to_string_lossy().to_string();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "swarm-backtrace",
            "land",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--run-dir",
            &run_dir,
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown field `proposal`"));
}

#[test]
fn swarm_backtrace_land_reports_nested_stable_id_errors() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let run_dir = dir.path().join("run");
    init_repo(&repo);
    create_source(&repo);
    write_bad_nested_stable_id(&run_dir);
    let run_dir = run_dir.to_string_lossy().to_string();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "swarm-backtrace",
            "land",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--run-dir",
            &run_dir,
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("stable id"))
        .stderr(predicates::str::contains("evidence/auth"));
}
