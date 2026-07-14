use super::support::{create_source, init_repo};
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

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
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "proposal already exists and is immutable",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated finding."))
        .stdout(predicates::str::contains("Updated synthesis."))
        .stdout(predicates::str::contains("Original proposal"))
        .stdout(predicates::str::contains(r#""confidence": 0.5"#));
}

#[test]
#[allow(clippy::too_many_lines)]
fn proposal_replace_refuses_accepted_human_disposition() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    init_repo(&repo);
    create_source(&repo, "source_codebase");
    let ideation = dir.path().join(".provenance/state/scopes/default/ideation");
    std::fs::create_dir_all(&ideation).unwrap();
    std::fs::write(
        ideation.join("contributions.jsonl"),
        concat!(r#"{"schema_version":1,"scope_id":"default","id":"contribution_publish","target":{"artifact_type":"source","artifact_id":"source_codebase"},"participant_slot":"extractor","stance":"support","strongest_finding":"Observed","evidence_references":[{"reference_id":"evidence_publish","evidence_type":"source","summary":"Pinned"}],"material_claims":[{"claim_id":"claim_publish","statement":"Observed","evidence_type":"source","evidence_reference_ids":["evidence_publish"]}],"risks":[],"objections":[],"challenges":[],"suggested_artifact_changes":[],"unsupported_recommendations":[],"uncertainty":{"level":"low","rationale":"Direct"},"open_questions":[]}"#, "\n"),
    ).unwrap();
    std::fs::write(
        ideation.join("synthesis_packets.jsonl"),
        concat!(r#"{"schema_version":1,"scope_id":"default","id":"synthesis_publish","target":{"artifact_type":"source","artifact_id":"source_codebase"},"summary":"Adjudicated","consensus":[],"contested_claims":[],"minority_objections":[],"evidence_gaps":[],"unsupported_speculation":[],"open_questions":[],"suggested_artifacts":[{"proposal_key":"backtrace/auth/publish_requires_worker","proposal_type":"requirement_candidate","summary":"Candidate","origin_participant_slots":["extractor"]}],"required_human_decisions":[]}"#, "\n"),
    ).unwrap();

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
            "--supporting-claim-id",
            "claim_publish",
            "--format",
            "json",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "assert",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "assertion_publish",
            "--proposal-id",
            "prop_req_publish_requires_worker",
            "--synthesis-packet-id",
            "synthesis_publish",
            "--supporting-claim-id",
            "claim_publish",
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
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "proposal already exists and is immutable",
        ));

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
