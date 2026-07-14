use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

fn proposal(repo: &str, id: &str, builds_on: Option<&str>) {
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
        "--supporting-claim-id",
        "claim_a",
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

    let ideation = dir.path().join(".provenance/state/scopes/default/ideation");
    std::fs::create_dir_all(&ideation).unwrap();
    std::fs::write(ideation.join("contributions.jsonl"), concat!(r#"{"schema_version":1,"scope_id":"default","id":"contribution_a","target":{"artifact_type":"requirement","artifact_id":"req_anchor"},"participant_slot":"extractor","stance":"support","strongest_finding":"Observed","evidence_references":[{"reference_id":"evidence_a","evidence_type":"source","summary":"Pinned"}],"material_claims":[{"claim_id":"claim_a","statement":"Observed","evidence_type":"source","evidence_reference_ids":["evidence_a"]}],"risks":[],"objections":[],"challenges":[],"suggested_artifact_changes":[],"unsupported_recommendations":[],"uncertainty":{"level":"low","rationale":"Direct"},"open_questions":[]}"#, "\n")).unwrap();
    std::fs::write(ideation.join("synthesis_packets.jsonl"), concat!(r#"{"schema_version":1,"scope_id":"default","id":"synthesis_a","target":{"artifact_type":"requirement","artifact_id":"req_anchor"},"summary":"Adjudicated","consensus":[],"contested_claims":[],"minority_objections":[],"evidence_gaps":[],"unsupported_speculation":[],"open_questions":[],"suggested_artifacts":[{"proposal_id":"proposal_asserted","proposal_key":"proposal_asserted","proposal_type":"requirement_candidate","summary":"Candidate","origin_participant_slots":["extractor"]}],"required_human_decisions":[]}"#, "\n")).unwrap();
    proposal(&repo, "proposal_asserted", None);
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
            "assertion_asserted",
            "--proposal-id",
            "proposal_asserted",
            "--synthesis-packet-id",
            "synthesis_a",
            "--supporting-claim-id",
            "claim_a",
        ])
        .assert()
        .success();
    proposal(&repo, "proposal_derivative", Some("assertion_asserted"));

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
            "builds on provisionally: assertion_asserted",
        ));
}
