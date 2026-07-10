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

pub(super) fn create_source(repo: &str) {
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

pub(super) fn write_run_dir(root: &std::path::Path, strongest_finding: &str) {
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
