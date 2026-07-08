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

fn create_source(repo: &str, id: &str) {
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

#[test]
#[allow(clippy::too_many_lines)]
fn cli_creates_materializes_and_exports_ideation_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    let export_path = dir.path().join("export.json").to_string_lossy().to_string();

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
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "sources",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "source_schads",
            "--name",
            "SCHADS Award",
            "--source-type",
            "policy",
            "--format",
            "json",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "requirements",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "req_overtime",
            "--statement",
            "Overtime must be traceable",
            "--format",
            "json",
        ])
        .assert()
        .success();

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
            "contrib_reviewer_001",
            "--target-type",
            "requirement",
            "--target-id",
            "req_overtime",
            "--participant-slot",
            "reviewer",
            "--stance",
            "support",
            "--strongest-finding",
            "The requirement is supported by source and code evidence.",
            "--evidence-json",
            r#"[{"reference_id":"evidence_code_line","evidence_type":"artifact","summary":"Existing payroll check","file_path":"src/payroll/overtime.rs","line":42}]"#,
            "--claims-json",
            r#"[{"claim_id":"claim_overtime_threshold","statement":"Overtime starts after the award threshold.","evidence_type":"artifact","evidence_reference_ids":["evidence_code_line"],"confidence":0.87}]"#,
            "--risks-json",
            r#"["Payroll underpayment if the threshold is wrong"]"#,
            "--uncertainty-level",
            "medium",
            "--uncertainty-rationale",
            "Agreement overrides were not reviewed.",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("contrib_reviewer_001"))
        .stdout(predicates::str::contains(r#""confidence": 0.87"#));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "synthesis-packets",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "synth_overtime_001",
            "--target-type",
            "requirement",
            "--target-id",
            "req_overtime",
            "--summary",
            "Participants agree the threshold needs explicit traceability.",
            "--consensus-json",
            r#"[{"statement":"The requirement needs a source reference.","supporting_participant_slots":["reviewer"],"evidence_reference_ids":["evidence_code_line"]}]"#,
            "--evidence-gaps-json",
            r#"[{"question":"Which agreement applies?","needed_evidence_type":"source","blocking_promotion":true}]"#,
            "--suggested-artifacts-json",
            r#"[{"proposal_key":"req-overtime-traceability","proposal_type":"requirement_candidate","summary":"Clarify source traceability.","origin_participant_slots":["reviewer"]}]"#,
            "--required-human-decisions-json",
            r#"[{"decision_key":"decide_agreement_scope","prompt":"Confirm the governing agreement.","blocks_promotion":true}]"#,
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("synth_overtime_001"));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "proposal_overtime_traceability",
            "--proposal-key",
            "req-overtime-traceability",
            "--proposal-type",
            "requirement_candidate",
            "--title",
            "Clarify overtime traceability",
            "--summary",
            "Add source-backed threshold language.",
            "--confidence",
            "0.83",
            "--target-type",
            "requirement",
            "--target-id",
            "req_overtime",
            "--source-id",
            "source_schads",
            "--evidence-json",
            r#"[{"reference_id":"evidence_code_line","evidence_type":"artifact","summary":"Existing payroll check","file_path":"src/payroll/overtime.rs","line":42}]"#,
            "--supporting-claim-id",
            "claim_overtime_threshold",
            "--promotion-state",
            "proposed",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("proposal_overtime_traceability"))
        .stdout(predicates::str::contains(r#""confidence": 0.83"#));

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
            "decision_overtime_traceability",
            "--proposal-id",
            "proposal_overtime_traceability",
            "--decision",
            "accepted",
            "--rationale",
            "Human confirmed the source traceability.",
            "--actor-id",
            "ben",
            "--actor-type",
            "human",
            "--actor-name",
            "Ben",
            "--canonical-artifact-type",
            "requirement",
            "--canonical-artifact-id",
            "req_overtime",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("decision_overtime_traceability"));

    Command::cargo_bin("provenance")
        .unwrap()
        .args(["materialize", "--repo", &repo, "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""records_loaded": 6"#));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "list",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("src/payroll/overtime.rs"))
        .stdout(predicates::str::contains(r#""confidence": 0.83"#))
        .stdout(predicates::str::contains(
            r#""promotion_state": "accepted""#,
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--format",
            "json",
            "--output",
            &export_path,
        ])
        .assert()
        .success();

    let exported = std::fs::read_to_string(export_path).unwrap();
    assert!(exported.contains(r#""proposal_cards""#));
    assert!(exported.contains(r#""confidence": 0.87"#));
    assert!(exported.contains(r#""confidence": 0.83"#));
    assert!(exported.contains(r#""promotion_decisions""#));
    assert!(exported.contains(r#""contributions""#));
    assert!(exported.contains(r#""synthesis_packets""#));
    assert!(exported.contains(r#""blocking_promotion": true"#));
}

#[test]
fn ideation_json_flags_accept_at_file_payloads() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    init_repo(&repo);
    create_source(&repo, "source_codebase");

    let evidence_path = dir.path().join("evidence.json");
    let claims_path = dir.path().join("claims.json");
    std::fs::write(
        &evidence_path,
        r#"[{"reference_id":"evidence_auth_guard","evidence_type":"artifact","summary":"Guard rejects missing worker","file_path":"src/auth.rs","line":12}]"#,
    )
    .unwrap();
    std::fs::write(
        &claims_path,
        r#"[{"claim_id":"claim_auth_guard","statement":"Publishing requires an assigned worker.","evidence_type":"artifact","evidence_reference_ids":["evidence_auth_guard"],"confidence":0.91}]"#,
    )
    .unwrap();
    let evidence_arg = format!("@{}", evidence_path.to_string_lossy());
    let claims_arg = format!("@{}", claims_path.to_string_lossy());

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
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Publishing is guarded by worker assignment.",
            "--evidence-json",
            &evidence_arg,
            "--claims-json",
            &claims_arg,
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Direct guard evidence.",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("claim_auth_guard"))
        .stdout(predicates::str::contains("evidence_auth_guard"));
}

#[test]
fn ideation_nested_ids_are_semantically_validated() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    init_repo(&repo);
    create_source(&repo, "source_codebase");

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
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Publishing is guarded by worker assignment.",
            "--evidence-json",
            r#"[{"reference_id":"evidence/auth","evidence_type":"artifact","summary":"Bad id","file_path":"src/auth.rs","line":12}]"#,
            "--claims-json",
            r#"[{"claim_id":"claim_auth_guard","statement":"Publishing requires an assigned worker.","evidence_type":"artifact","evidence_reference_ids":["evidence/auth"]}]"#,
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Direct guard evidence.",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("stable id"));
}

#[test]
#[allow(clippy::too_many_lines)]
fn ideation_create_replace_updates_existing_records() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    init_repo(&repo);
    create_source(&repo, "source_codebase");

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
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Original finding.",
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Initial run.",
            "--format",
            "json",
        ])
        .assert()
        .success();
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
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Duplicate finding.",
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Duplicate run.",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
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
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Updated finding.",
            "--uncertainty-level",
            "medium",
            "--uncertainty-rationale",
            "Second run.",
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated finding."));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "synthesis-packets",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "synth_backtrace_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--summary",
            "Original synthesis.",
            "--format",
            "json",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "synthesis-packets",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "synth_backtrace_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--summary",
            "Updated synthesis.",
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated synthesis."));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "prop_req_publish_requires_worker",
            "--proposal-key",
            "backtrace/auth/publish_requires_worker",
            "--proposal-type",
            "requirement_candidate",
            "--title",
            "Original proposal",
            "--summary",
            "Original summary.",
            "--confidence",
            "0.5",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--source-id",
            "source_codebase",
            "--promotion-state",
            "proposed",
            "--format",
            "json",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "prop_req_publish_requires_worker",
            "--proposal-key",
            "backtrace/auth/publish_requires_worker",
            "--proposal-type",
            "requirement_candidate",
            "--title",
            "Updated proposal",
            "--summary",
            "Updated summary.",
            "--confidence",
            "0.9",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--source-id",
            "source_codebase",
            "--promotion-state",
            "proposed",
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated proposal"))
        .stdout(predicates::str::contains(r#""confidence": 0.9"#));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated finding."))
        .stdout(predicates::str::contains("Updated synthesis."))
        .stdout(predicates::str::contains("Updated proposal"))
        .stdout(predicates::str::contains(r#""confidence": 0.9"#));
}

#[test]
#[allow(clippy::too_many_lines)]
fn proposal_replace_refuses_accepted_human_disposition() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    init_repo(&repo);
    create_source(&repo, "source_codebase");

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "prop_req_publish_requires_worker",
            "--proposal-key",
            "backtrace/auth/publish_requires_worker",
            "--proposal-type",
            "requirement_candidate",
            "--title",
            "Publishing requires an assigned worker",
            "--summary",
            "Candidate requirement extracted from the publishing guard.",
            "--confidence",
            "0.91",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--source-id",
            "source_codebase",
            "--promotion-state",
            "proposed",
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
            "Human accepted the proposed requirement.",
            "--actor-id",
            "ben",
            "--actor-type",
            "human",
            "--format",
            "json",
        ])
        .assert()
        .success();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "prop_req_publish_requires_worker",
            "--proposal-key",
            "backtrace/auth/publish_requires_worker",
            "--proposal-type",
            "requirement_candidate",
            "--title",
            "Replacement proposal",
            "--summary",
            "This must not overwrite a human decision.",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--source-id",
            "source_codebase",
            "--promotion-state",
            "proposed",
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
            "proposals",
            "list",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            r#""promotion_state": "accepted""#,
        ))
        .stdout(predicates::str::contains("Replacement proposal").not());
}
