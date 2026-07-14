use crate::{
    validate_ideation_aggregate, AssertionRecord, Contribution, IdeationAggregate, ProposalCard,
    SynthesisPacket,
};

fn aggregate(
    contribution: serde_json::Value,
    synthesis: serde_json::Value,
    proposals: Vec<serde_json::Value>,
    assertions: Vec<serde_json::Value>,
) -> anyhow::Result<()> {
    let contributions = vec![serde_json::from_value::<Contribution>(contribution).unwrap()];
    let synthesis_packets = vec![serde_json::from_value::<SynthesisPacket>(synthesis).unwrap()];
    let proposals = proposals
        .into_iter()
        .map(|value| serde_json::from_value::<ProposalCard>(value).unwrap())
        .collect::<Vec<_>>();
    let assertions = assertions
        .into_iter()
        .map(|value| serde_json::from_value::<AssertionRecord>(value).unwrap())
        .collect::<Vec<_>>();
    validate_ideation_aggregate(IdeationAggregate {
        contributions: &contributions,
        synthesis_packets: &synthesis_packets,
        proposals: &proposals,
        assertions: &assertions,
        dispositions: &[],
    })
}

fn fixtures() -> (
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
) {
    let contribution = serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "contribution_a",
        "target": {"artifact_type": "requirement", "artifact_id": "req_a"},
        "participant_slot": "extractor", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_a", "evidence_type": "source", "summary": "Pinned code"}],
        "material_claims": [{"claim_id": "claim_a", "statement": "Behavior exists", "evidence_type": "source", "evidence_reference_ids": ["evidence_a"], "confidence": 0.9}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    });
    let synthesis = serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "synthesis_a",
        "target": {"artifact_type": "requirement", "artifact_id": "req_a"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_key": "proposal-a", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["extractor"]}],
        "required_human_decisions": []
    });
    let proposal = serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "proposal_a", "proposal_key": "proposal-a",
        "proposal_type": "requirement_candidate", "title": "Candidate", "summary": "Candidate",
        "traceability": {"target": {"artifact_type": "requirement", "artifact_id": "req_a"}, "source_ids": [], "evidence_references": [], "supporting_claim_ids": ["claim_a"]},
        "promotion_state": "proposed"
    });
    let assertion = serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "assertion_a", "proposal_id": "proposal_a",
        "synthesis_packet_id": "synthesis_a", "supporting_claim_ids": ["claim_a"]
    });
    (contribution, synthesis, proposal, assertion)
}

#[test]
fn assertion_requires_owned_positive_adjudication_evidence() {
    let (contribution, synthesis, proposal, assertion) = fixtures();
    aggregate(
        contribution.clone(),
        synthesis.clone(),
        vec![proposal.clone()],
        vec![assertion.clone()],
    )
    .unwrap();

    let mut missing = assertion.clone();
    missing["supporting_claim_ids"] = serde_json::json!(["claim_missing"]);
    assert!(aggregate(
        contribution.clone(),
        synthesis.clone(),
        vec![proposal.clone()],
        vec![missing]
    )
    .unwrap_err()
    .to_string()
    .contains("claim_missing does not exist"));

    let mut foreign = contribution.clone();
    foreign["target"]["artifact_id"] = serde_json::json!("req_other");
    assert!(aggregate(
        foreign,
        synthesis.clone(),
        vec![proposal.clone()],
        vec![assertion.clone()]
    )
    .unwrap_err()
    .to_string()
    .contains("owned by the proposal target"));

    let mut no_evidence = contribution;
    no_evidence["material_claims"][0]["evidence_reference_ids"] = serde_json::json!([]);
    assert!(
        aggregate(no_evidence, synthesis, vec![proposal], vec![assertion])
            .unwrap_err()
            .to_string()
            .contains("positive evidence")
    );
}

#[test]
fn assertion_rejects_contested_claims_and_blockers() {
    let (contribution, synthesis, proposal, assertion) = fixtures();
    let mut contested = synthesis.clone();
    contested["contested_claims"] = serde_json::json!([{"claim_id": "claim_a", "statement": "Disputed", "supporting_participant_slots": [], "opposing_participant_slots": ["refuter"], "evidence_quality": "weak"}]);
    assert!(aggregate(
        contribution.clone(),
        contested,
        vec![proposal.clone()],
        vec![assertion.clone()]
    )
    .unwrap_err()
    .to_string()
    .contains("contested"));

    let mut gap = synthesis.clone();
    gap["evidence_gaps"] = serde_json::json!([{"question": "Unknown", "needed_evidence_type": "source", "blocking_promotion": true}]);
    assert!(aggregate(
        contribution.clone(),
        gap,
        vec![proposal.clone()],
        vec![assertion.clone()]
    )
    .unwrap_err()
    .to_string()
    .contains("blocking evidence gap"));

    let mut human = synthesis;
    human["required_human_decisions"] = serde_json::json!([{"decision_key": "decision_needed", "prompt": "Choose", "blocks_promotion": true}]);
    assert!(
        aggregate(contribution, human, vec![proposal], vec![assertion])
            .unwrap_err()
            .to_string()
            .contains("blocking human decision")
    );
}

#[test]
fn assertion_lineage_uses_immutable_assertion_ids_and_is_acyclic() {
    let (contribution, synthesis, proposal, assertion) = fixtures();
    let mut child = proposal.clone();
    child["id"] = serde_json::json!("proposal_child");
    child["proposal_key"] = serde_json::json!("proposal-child");
    child["builds_on"] = serde_json::json!(["assertion_missing"]);
    assert!(aggregate(
        contribution.clone(),
        synthesis.clone(),
        vec![proposal.clone(), child.clone()],
        vec![assertion.clone()]
    )
    .unwrap_err()
    .to_string()
    .contains("assertion_missing does not exist"));

    let mut assertion_child = assertion.clone();
    assertion_child["id"] = serde_json::json!("assertion_child");
    assertion_child["proposal_id"] = serde_json::json!("proposal_child");
    child["builds_on"] = serde_json::json!(["assertion_a"]);
    let mut parent = proposal;
    parent["builds_on"] = serde_json::json!(["assertion_child"]);
    let mut synthesis = synthesis;
    synthesis["suggested_artifacts"].as_array_mut().unwrap().push(serde_json::json!({"proposal_key": "proposal-child", "proposal_type": "requirement_candidate", "summary": "Child", "origin_participant_slots": ["extractor"]}));
    assert!(aggregate(
        contribution,
        synthesis,
        vec![child, parent],
        vec![assertion_child, assertion]
    )
    .unwrap_err()
    .to_string()
    .contains("cycle"));
}
