use super::support::{create_source, init_repo, write_run_dir};
use assert_cmd::Command;

#[test]
fn swarm_backtrace_land_writes_run_dir_outputs_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let run_dir = dir.path().join("run");
    init_repo(&repo);
    create_source(&repo);
    write_run_dir(&run_dir, "Publishing is guarded by worker assignment.");
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
        .success()
        .stdout(predicates::str::contains(r#""contributions": 2"#))
        .stdout(predicates::str::contains(r#""synthesis_packets": 1"#))
        .stdout(predicates::str::contains(r#""proposals": 1"#));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("contrib_backtrace_extract_auth"))
        .stdout(predicates::str::contains("synth_backtrace_auth"))
        .stdout(predicates::str::contains(
            "prop_req_publish_requires_worker",
        ))
        .stdout(predicates::str::contains(r#""confidence": 0.91"#));
}

#[test]
fn swarm_backtrace_land_can_replace_existing_run_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let run_dir = dir.path().join("run");
    init_repo(&repo);
    create_source(&repo);
    write_run_dir(&run_dir, "Original extracted finding.");
    let run_dir_arg = run_dir.to_string_lossy().to_string();

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
            &run_dir_arg,
            "--format",
            "json",
        ])
        .assert()
        .success();
    write_run_dir(&run_dir, "Updated extracted finding.");
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
            &run_dir_arg,
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""replace": true"#));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated extracted finding."))
        .stdout(predicates::str::contains(r#""contributions""#));
}
