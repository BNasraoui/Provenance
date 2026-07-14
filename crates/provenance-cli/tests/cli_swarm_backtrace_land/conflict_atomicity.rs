use super::support::{create_source, init_repo, write_run_dir};
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

fn write_duplicate_contribution(root: &std::path::Path) {
    let contributions = root.join("contributions");
    std::fs::create_dir_all(&contributions).unwrap();
    std::fs::write(
        contributions.join("duplicate.json"),
        r#"{
          "contribution": {
            "schema_version": 1,
            "scope_id": "default",
            "id": "contrib_backtrace_extract_auth",
            "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
            "participant_slot": "duplicate_extract_auth",
            "stance": "support",
            "strongest_finding": "Duplicate contribution id in the same run.",
            "evidence_references": [],
            "material_claims": [],
            "risks": [],
            "objections": [],
            "challenges": [],
            "suggested_artifact_changes": [],
            "unsupported_recommendations": [],
            "uncertainty": {"level":"low","rationale":"Duplicate id fixture."},
            "open_questions": []
          }
        }"#,
    )
    .unwrap();
}

#[test]
fn swarm_backtrace_land_rejects_duplicate_run_ids_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let run_dir = dir.path().join("run");
    init_repo(&repo);
    create_source(&repo);
    write_run_dir(&run_dir, "Publishing is guarded by worker assignment.");
    write_duplicate_contribution(&run_dir);
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
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "duplicate contribution id contrib_backtrace_extract_auth in run",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("contrib_backtrace_extract_auth").not())
        .stdout(predicates::str::contains("synth_backtrace_auth").not())
        .stdout(predicates::str::contains("prop_req_publish_requires_worker").not());
}

#[test]
fn swarm_backtrace_land_rejects_existing_ids_before_writing() {
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
            "contributions",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "contrib_backtrace_refute_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "preexisting_refute_auth",
            "--stance",
            "mixed",
            "--strongest-finding",
            "Pre-existing refuter finding.",
            "--uncertainty-level",
            "medium",
            "--uncertainty-rationale",
            "Seeded existing record.",
            "--format",
            "json",
        ])
        .assert()
        .success();

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
            "contribution contrib_backtrace_refute_auth already exists",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Pre-existing refuter finding."))
        .stdout(predicates::str::contains("contrib_backtrace_extract_auth").not())
        .stdout(predicates::str::contains("synth_backtrace_auth").not())
        .stdout(predicates::str::contains("prop_req_publish_requires_worker").not());
}

#[test]
fn swarm_backtrace_land_refuses_to_replace_accepted_proposals() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let run_dir = dir.path().join("run");
    init_repo(&repo);
    create_source(&repo);
    write_run_dir(&run_dir, "Original extracted finding.");
    let merge_path = run_dir.join("merge/merged.json");
    let mut merge: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&merge_path).unwrap()).unwrap();
    merge["synthesis_packet"]["contested_claims"] = serde_json::json!([]);
    merge["synthesis_packet"]["required_human_decisions"] = serde_json::json!([]);
    merge["assertions"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "assertion_publish_guard",
        "proposal_id": "prop_req_publish_requires_worker", "synthesis_packet_id": "synth_backtrace_auth",
        "supporting_claim_ids": ["claim_auth_guard"]
    }]);
    std::fs::write(&merge_path, serde_json::to_vec_pretty(&merge).unwrap()).unwrap();
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
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "promotion-decisions",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "decision_publish_requires_worker",
            "--proposal-id",
            "prop_req_publish_requires_worker",
            "--decision",
            "accepted",
            "--rationale",
            "Human accepted the proposal.",
            "--actor-id",
            "ben",
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
        .failure()
        .stderr(predicates::str::contains(
            "durable assertion evidence and cannot be replaced",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Original extracted finding."))
        .stdout(predicates::str::contains("Updated extracted finding.").not())
        .stdout(predicates::str::contains(
            "decision_publish_requires_worker",
        ));
}
