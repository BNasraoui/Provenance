use assert_cmd::Command;

fn write_json(dir: &tempfile::TempDir, name: &str, json: &str) -> String {
    let path = dir.path().join(name);
    std::fs::write(&path, json).unwrap();
    path.to_string_lossy().to_string()
}

#[test]
fn schema_show_outputs_ideation_artifact_schemas() {
    for (artifact, expected_title) in [
        ("contribution", "Contribution"),
        ("synthesis-packet", "SynthesisPacket"),
        ("proposal", "ProposalCard"),
    ] {
        Command::cargo_bin("provenance")
            .unwrap()
            .args(["schema", "show", artifact, "--format", "json"])
            .assert()
            .success()
            .stdout(predicates::str::contains(expected_title))
            .stdout(predicates::str::contains("stableId"))
            .stdout(predicates::str::contains("^[a-z0-9_-]+$"));
    }
}

#[test]
fn validate_accepts_good_ideation_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let contribution = write_json(
        &dir,
        "contribution.json",
        r#"{
          "schema_version": 1,
          "scope_id": "default",
          "id": "contrib_extract_auth",
          "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
          "participant_slot": "extract_auth",
          "stance": "support",
          "strongest_finding": "Publishing is guarded by worker assignment.",
          "evidence_references": [{"reference_id":"evidence_auth_guard","evidence_type":"artifact","summary":"Guard rejects missing worker","file_path":"src/auth.rs","line":12}],
          "material_claims": [{"claim_id":"claim_auth_guard","statement":"Publishing requires an assigned worker.","evidence_type":"artifact","evidence_reference_ids":["evidence_auth_guard"],"confidence":0.91}],
          "risks": [],
          "objections": [],
          "challenges": [],
          "suggested_artifact_changes": [],
          "unsupported_recommendations": [],
          "uncertainty": {"level":"low","rationale":"Direct guard evidence."},
          "open_questions": []
        }"#,
    );
    let synthesis = write_json(
        &dir,
        "synthesis.json",
        r#"{
          "schema_version": 1,
          "scope_id": "default",
          "id": "synth_backtrace_auth",
          "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
          "summary": "Extractor and refuter agree on the guard.",
          "consensus": [{"statement":"Publishing is guarded.","supporting_participant_slots":["extract_auth"],"evidence_reference_ids":["evidence_auth_guard"]}],
          "contested_claims": [],
          "minority_objections": [],
          "evidence_gaps": [],
          "unsupported_speculation": [],
          "open_questions": [],
          "suggested_artifacts": [{"proposal_key":"backtrace/auth/publish_requires_worker","proposal_type":"requirement_candidate","summary":"Review the candidate requirement.","origin_participant_slots":["extract_auth"]}],
          "required_human_decisions": [{"decision_key":"decide_publish_guard","prompt":"Confirm this behavior is intentional.","blocks_promotion":true}]
        }"#,
    );
    let proposal = write_json(
        &dir,
        "proposal.json",
        r#"{
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
        }"#,
    );

    for (artifact, path) in [
        ("contribution", contribution),
        ("synthesis-packet", synthesis),
        ("proposal", proposal),
    ] {
        Command::cargo_bin("provenance")
            .unwrap()
            .args(["validate", artifact, "--input", &path, "--format", "json"])
            .assert()
            .success()
            .stdout(predicates::str::contains(r#""valid": true"#))
            .stdout(predicates::str::contains(artifact));
    }
}

#[test]
fn validate_rejects_nested_invalid_stable_ids() {
    let dir = tempfile::tempdir().unwrap();
    let contribution = write_json(
        &dir,
        "bad-contribution.json",
        r#"{
          "schema_version": 1,
          "scope_id": "default",
          "id": "contrib_extract_auth",
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
        }"#,
    );
    let synthesis = write_json(
        &dir,
        "bad-synthesis.json",
        r#"{
          "schema_version": 1,
          "scope_id": "default",
          "id": "synth_backtrace_auth",
          "target": {"artifact_type": "source", "artifact_id": "source_codebase"},
          "summary": "Extractor and refuter agree on the guard.",
          "consensus": [],
          "contested_claims": [],
          "minority_objections": [],
          "evidence_gaps": [],
          "unsupported_speculation": [],
          "open_questions": [],
          "suggested_artifacts": [],
          "required_human_decisions": [{"decision_key":"decide/publish_guard","prompt":"Confirm this behavior is intentional.","blocks_promotion":true}]
        }"#,
    );
    let proposal = write_json(
        &dir,
        "bad-proposal.json",
        r#"{
          "schema_version": 1,
          "scope_id": "default",
          "id": "prop_req_publish_requires_worker",
          "proposal_key": "backtrace/auth/publish_requires_worker",
          "proposal_type": "requirement_candidate",
          "title": "Publishing requires an assigned worker",
          "summary": "Candidate requirement extracted from the publishing guard.",
          "traceability": {
            "target": {"artifact_type": "source", "artifact_id": "source/codebase"},
            "source_ids": ["source/codebase"],
            "evidence_references": [],
            "supporting_claim_ids": []
          },
          "promotion_state": "proposed"
        }"#,
    );

    for (artifact, path) in [
        ("contribution", contribution),
        ("synthesis-packet", synthesis),
        ("proposal", proposal),
    ] {
        Command::cargo_bin("provenance")
            .unwrap()
            .args(["validate", artifact, "--input", &path, "--format", "json"])
            .assert()
            .failure()
            .stderr(predicates::str::contains("stable id"));
    }
}
