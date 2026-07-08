use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

fn init_repo(repo: &str) {
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

fn create_source(repo: &str) {
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
            "source_codebase",
            "--name",
            "Example @ abc1234",
            "--source-type",
            "system_state",
            "--reference",
            "git:example@abc1234",
            "--commit-pin",
            "abc1234",
            "--format",
            "json",
        ])
        .assert()
        .success();
}

fn write_run_dir(root: &std::path::Path, strongest_finding: &str) {
    let extractors = root.join("extractors");
    let refuters = root.join("refuters");
    let merge = root.join("merge");
    std::fs::create_dir_all(&extractors).unwrap();
    std::fs::create_dir_all(&refuters).unwrap();
    std::fs::create_dir_all(&merge).unwrap();

    std::fs::write(
        extractors.join("auth.json"),
        format!(
            r#"{{
              "contribution": {{
                "schema_version": 1,
                "scope_id": "default",
                "id": "contrib_backtrace_extract_auth",
                "target": {{"artifact_type": "source", "artifact_id": "source_codebase"}},
                "participant_slot": "extract_auth",
                "stance": "support",
                "strongest_finding": "{strongest_finding}",
                "evidence_references": [{{"reference_id":"evidence_auth_guard","evidence_type":"artifact","summary":"Guard rejects missing worker","file_path":"src/auth.rs","line":12}}],
                "material_claims": [{{"claim_id":"claim_auth_guard","statement":"Publishing requires an assigned worker.","evidence_type":"artifact","evidence_reference_ids":["evidence_auth_guard"],"confidence":0.91}}],
                "risks": [],
                "objections": [],
                "challenges": [],
                "suggested_artifact_changes": [],
                "unsupported_recommendations": [],
                "uncertainty": {{"level":"low","rationale":"Direct guard evidence."}},
                "open_questions": []
              }}
            }}"#
        ),
    )
    .unwrap();
    std::fs::write(
        refuters.join("auth.json"),
        r#"{
          "contribution": {
            "schema_version": 1,
            "scope_id": "default",
            "id": "contrib_backtrace_refute_auth",
            "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
            "participant_slot": "refute_auth",
            "stance": "mixed",
            "strongest_finding": "The guard is real, but intent still needs confirmation.",
            "evidence_references": [{"reference_id":"evidence_auth_guard","evidence_type":"artifact","summary":"Guard rejects missing worker","file_path":"src/auth.rs","line":12}],
            "material_claims": [],
            "risks": [],
            "objections": ["Intent is inferred from enforcement only."],
            "challenges": [{"claim_id":"claim_auth_guard","objection":"Code proves enforcement, not product intent."}],
            "suggested_artifact_changes": [],
            "unsupported_recommendations": [],
            "uncertainty": {"level":"medium","rationale":"Intent requires human confirmation."},
            "open_questions": ["Is this guard intentional product behavior?"]
          }
        }"#,
    )
    .unwrap();
    std::fs::write(
        merge.join("merged.json"),
        r#"{
          "synthesis_packet": {
            "schema_version": 1,
            "scope_id": "default",
            "id": "synth_backtrace_auth",
            "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
            "summary": "Extractor and refuter agree that publishing is guarded.",
            "consensus": [{"statement":"Publishing is guarded by worker assignment.","supporting_participant_slots":["extract_auth","refute_auth"],"evidence_reference_ids":["evidence_auth_guard"]}],
            "contested_claims": [{"claim_id":"claim_auth_guard","statement":"Publishing requires an assigned worker.","supporting_participant_slots":["extract_auth"],"opposing_participant_slots":["refute_auth"],"evidence_quality":"strong"}],
            "minority_objections": [{"participant_slot":"refute_auth","objection":"Intent still needs confirmation.","evidence_reference_ids":["evidence_auth_guard"]}],
            "evidence_gaps": [],
            "unsupported_speculation": [],
            "open_questions": [],
            "suggested_artifacts": [{"proposal_key":"backtrace/auth/publish_requires_worker","proposal_type":"requirement_candidate","summary":"Review the candidate requirement.","origin_participant_slots":["extract_auth"]}],
            "required_human_decisions": [{"decision_key":"decide_publish_guard","prompt":"Confirm this behavior is intentional.","blocks_promotion":true}]
          },
          "proposals": [{
            "schema_version": 1,
            "scope_id": "default",
            "id": "prop_req_publish_requires_worker",
            "proposal_key": "backtrace/auth/publish_requires_worker",
            "proposal_type": "requirement_candidate",
            "title": "Publishing requires an assigned worker",
            "summary": "Candidate requirement extracted from the publishing guard.",
            "confidence": 0.91,
            "traceability": {
              "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
              "source_ids": ["source_codebase"],
              "evidence_references": [{"reference_id":"evidence_auth_guard","evidence_type":"artifact","summary":"Guard rejects missing worker","file_path":"src/auth.rs","line":12}],
              "supporting_claim_ids": ["claim_auth_guard"]
            },
            "promotion_state": "proposed"
          }]
        }"#,
    )
    .unwrap();
}

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

fn write_bad_nested_stable_id(root: &std::path::Path) {
    let extractors = root.join("extractors");
    let merge = root.join("merge");
    std::fs::create_dir_all(&extractors).unwrap();
    std::fs::create_dir_all(&merge).unwrap();
    std::fs::write(
        extractors.join("bad.json"),
        r#"{
          "contribution": {
            "schema_version": 1,
            "scope_id": "default",
            "id": "contrib_backtrace_extract_auth",
            "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
            "participant_slot": "extract_auth",
            "stance": "support",
            "strongest_finding": "Publishing is guarded by worker assignment.",
            "evidence_references": [{"reference_id":"evidence/auth","evidence_type":"artifact","summary":"Bad nested id"}],
            "material_claims": [],
            "risks": [],
            "objections": [],
            "challenges": [],
            "suggested_artifact_changes": [],
            "unsupported_recommendations": [],
            "uncertainty": {"level":"low","rationale":"Direct guard evidence."},
            "open_questions": []
          }
        }"#,
    )
    .unwrap();
    std::fs::write(
        merge.join("merged.json"),
        r#"{
          "synthesis_packet": {
            "schema_version": 1,
            "scope_id": "default",
            "id": "synth_backtrace_auth",
            "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
            "summary": "Extractor found a guard.",
            "consensus": [],
            "contested_claims": [],
            "minority_objections": [],
            "evidence_gaps": [],
            "unsupported_speculation": [],
            "open_questions": [],
            "suggested_artifacts": [],
            "required_human_decisions": []
          },
          "proposals": []
        }"#,
    )
    .unwrap();
}

fn write_bad_proposal_confidence(root: &std::path::Path) {
    write_run_dir(root, "Publishing is guarded by worker assignment.");
    let merge_path = root.join("merge").join("merged.json");
    let merge_json = std::fs::read_to_string(&merge_path).unwrap();
    std::fs::write(
        &merge_path,
        merge_json.replace(r#""confidence": 0.91"#, r#""confidence": 1.5"#),
    )
    .unwrap();
}

fn write_merge_output_with_unknown_key(root: &std::path::Path) {
    write_run_dir(root, "Publishing is guarded by worker assignment.");
    let merge_path = root.join("merge").join("merged.json");
    let merge_json = std::fs::read_to_string(&merge_path).unwrap();
    std::fs::write(
        &merge_path,
        merge_json.replace(r#""proposals": ["#, r#""proposal": ["#),
    )
    .unwrap();
}

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
            "--actor-type",
            "human",
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
        .stderr(predicates::str::contains("human disposition"))
        .stderr(predicates::str::contains("accepted"));

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
            r#""promotion_state": "accepted""#,
        ));
}
