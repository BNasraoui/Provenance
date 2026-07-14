use super::ideation::contributions::{Contribution, MaterialClaim};
use super::ideation::promotions::DispositionRecord;
use super::ideation::proposals::ProposalCard;
use super::ideation::synthesis::SynthesisPacket;
use super::ideation::{PromotionDecision, PromotionState, ProposalType};
use super::ids::SchemaVersion;

#[test]
#[allow(clippy::too_many_lines)]
fn ideation_output_records_roundtrip_without_schema_bump() {
    let contribution = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "contrib_reviewer_001",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
        "participant_slot": "reviewer",
        "stance": "support",
        "strongest_finding": "The requirement is supported by the award clause.",
        "evidence_references": [{
            "reference_id": "evidence_award_clause",
            "evidence_type": "source",
            "summary": "SCHADS overtime clause",
            "file_path": "src/payroll/overtime.rs",
            "line": 42
        }],
        "material_claims": [{
            "claim_id": "claim_overtime_threshold",
            "statement": "Overtime starts after the award threshold.",
            "evidence_type": "source",
            "evidence_reference_ids": ["evidence_award_clause"],
            "confidence": 0.87
        }],
        "risks": ["Payroll underpayment if the threshold is wrong"],
        "objections": [],
        "challenges": [],
        "suggested_artifact_changes": [{
            "artifact_type": "requirement",
            "artifact_id": "req_overtime",
            "change_type": "update",
            "supporting_claim_ids": ["claim_overtime_threshold"],
            "summary": "Clarify the threshold source."
        }],
        "unsupported_recommendations": [{
            "recommendation": "Check enterprise agreement overrides.",
            "marker": "exploratory"
        }],
        "uncertainty": {"level": "medium", "rationale": "Agreement overrides were not reviewed."},
        "open_questions": ["Does the agreement override SCHADS?"]
    });
    let synthesis = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "synth_overtime_001",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
        "summary": "Participants agree the threshold needs explicit traceability.",
        "consensus": [{
            "statement": "The requirement needs an award source reference.",
            "supporting_participant_slots": ["reviewer"],
            "evidence_reference_ids": ["evidence_award_clause"]
        }],
        "contested_claims": [{
            "claim_id": "claim_agreement_override",
            "statement": "The agreement overrides the award.",
            "supporting_participant_slots": [],
            "opposing_participant_slots": ["reviewer"],
            "evidence_quality": "unsupported"
        }],
        "minority_objections": [{
            "participant_slot": "reviewer",
            "objection": "Agreement coverage still needs checking.",
            "evidence_reference_ids": []
        }],
        "evidence_gaps": [{
            "question": "Which agreement applies?",
            "needed_evidence_type": "source",
            "blocking_promotion": true
        }],
        "unsupported_speculation": [{
            "statement": "The agreement probably matches SCHADS.",
            "originating_participant_slots": ["reviewer"],
            "marker": "unsupported"
        }],
        "open_questions": ["Which agreement applies?"],
        "suggested_artifacts": [{
            "proposal_key": "req-overtime-traceability",
            "proposal_type": "requirement_candidate",
            "summary": "Clarify source traceability.",
            "origin_participant_slots": ["reviewer"]
        }],
        "required_human_decisions": [{
            "decision_key": "decide_agreement_scope",
            "prompt": "Confirm the governing agreement.",
            "blocks_promotion": true
        }]
    });
    let proposal = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "proposal_overtime_traceability",
        "proposal_key": "req-overtime-traceability",
        "proposal_type": "requirement_candidate",
        "title": "Clarify overtime traceability",
        "summary": "Add source-backed threshold language.",
        "confidence": 0.83,
        "traceability": {
            "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
            "source_ids": ["source_schads"],
            "evidence_references": [{
                "reference_id": "evidence_code_line",
                "evidence_type": "artifact",
                "summary": "Existing payroll check",
                "file_path": "src/payroll/overtime.rs",
                "line": 42
            }],
            "supporting_claim_ids": ["claim_overtime_threshold"]
        },
        "promotion_state": "proposed"
    });
    let decision = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "decision_overtime_traceability",
        "proposal_id": "proposal_overtime_traceability",
        "decision": "accepted",
        "rationale": "Human confirmed the source traceability.",
        "actor": {"identity_type": "human", "id": "ben", "name": "Ben"},
        "canonical_artifact": {"artifact_type": "requirement", "artifact_id": "req_overtime"}
    });

    let contribution: Contribution = serde_json::from_value(contribution).unwrap();
    let synthesis: SynthesisPacket = serde_json::from_value(synthesis).unwrap();
    let proposal: ProposalCard = serde_json::from_value(proposal).unwrap();
    let decision: DispositionRecord = serde_json::from_value(decision).unwrap();

    assert_eq!(contribution.schema_version, SchemaVersion(1));
    assert_eq!(
        contribution.evidence_references[0].file_path.as_deref(),
        Some("src/payroll/overtime.rs")
    );
    assert!(synthesis.evidence_gaps[0].blocking_promotion);
    assert_eq!(
        proposal.traceability.source_ids[0].as_str(),
        "source_schads"
    );
    assert_eq!(proposal.promotion_state, PromotionState::Proposed);
    assert_eq!(decision.decision, PromotionDecision::Accepted);

    assert_eq!(
        serde_json::to_value(&contribution).unwrap()["schema_version"],
        1
    );
    assert_eq!(
        serde_json::to_value(&contribution).unwrap()["material_claims"][0]["confidence"],
        0.87
    );
    assert_eq!(
        serde_json::to_value(&synthesis).unwrap()["suggested_artifacts"][0]["proposal_type"],
        "requirement_candidate"
    );
    assert_eq!(serde_json::to_value(&proposal).unwrap()["confidence"], 0.83);
    assert_eq!(
        serde_json::to_value(&proposal).unwrap()["traceability"]["evidence_references"][0]["line"],
        42
    );
    assert_eq!(
        serde_json::to_value(&decision).unwrap()["actor"]["identity_type"],
        "human"
    );
}

#[test]
fn confidence_scores_must_be_in_unit_interval() {
    let claim = serde_json::json!({
        "claim_id": "claim_overtime_threshold",
        "statement": "Overtime starts after the award threshold.",
        "evidence_type": "source",
        "evidence_reference_ids": ["evidence_award_clause"],
        "confidence": 1.01
    });
    let proposal = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "proposal_overtime_traceability",
        "proposal_key": "req-overtime-traceability",
        "proposal_type": "requirement_candidate",
        "title": "Clarify overtime traceability",
        "summary": "Add source-backed threshold language.",
        "confidence": -0.01,
        "traceability": {
            "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
            "source_ids": ["source_schads"],
            "evidence_references": [],
            "supporting_claim_ids": ["claim_overtime_threshold"]
        },
        "promotion_state": "proposed"
    });

    assert!(serde_json::from_value::<MaterialClaim>(claim)
        .unwrap_err()
        .to_string()
        .contains("confidence"));
    assert!(serde_json::from_value::<ProposalCard>(proposal)
        .unwrap_err()
        .to_string()
        .contains("confidence"));
}

#[test]
fn proposal_and_promotion_decision_accept_convex_id_aliases() {
    let proposal = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "proposalId": "proposal_overtime_traceability",
        "proposalKey": "req-overtime-traceability",
        "proposalType": "requirement_candidate",
        "title": "Clarify overtime traceability",
        "summary": "Add source-backed threshold language.",
        "traceability": {
            "target": {"artifactType": "requirement", "artifactId": "req_overtime"},
            "sourceIds": ["source_schads"],
            "evidenceReferences": [{
                "referenceId": "evidence_code_line",
                "evidenceType": "artifact",
                "summary": "Existing payroll check",
                "filePath": "src/payroll/overtime.rs",
                "line": 42
            }],
            "supportingClaimIds": ["claim_overtime_threshold"]
        },
        "promotionState": "proposed"
    });
    let decision = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "promotionDecisionId": "decision_overtime_traceability",
        "proposalId": "proposal_overtime_traceability",
        "decision": "accepted",
        "rationale": "Human confirmed the source traceability.",
        "decidedBy": {"identityType": "human", "userId": "ben", "name": "Ben"},
        "canonicalArtifact": {"artifactType": "requirement", "artifactId": "req_overtime"}
    });

    let proposal: ProposalCard = serde_json::from_value(proposal).unwrap();
    let decision: DispositionRecord = serde_json::from_value(decision).unwrap();

    assert_eq!(proposal.id.as_str(), "proposal_overtime_traceability");
    assert_eq!(proposal.proposal_type, ProposalType::RequirementCandidate);
    assert_eq!(decision.id.as_str(), "decision_overtime_traceability");
    assert_eq!(
        decision.proposal_id.as_str(),
        "proposal_overtime_traceability"
    );
    assert_eq!(decision.actor.id, "ben");
}

#[test]
fn persisted_asserted_projection_round_trips_with_assertion_lineage() {
    let value = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "proposal_overtime_v2",
        "proposal_key": "overtime-v2",
        "proposal_type": "requirement_candidate",
        "title": "Clarify overtime",
        "summary": "Builds on the evidence-backed first pass.",
        "traceability": {
            "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
            "source_ids": [],
            "evidence_references": [],
            "supporting_claim_ids": []
        },
        "promotion_state": "asserted",
        "builds_on": ["assertion_overtime_v1"]
    });

    let proposal: ProposalCard = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(proposal.promotion_state, PromotionState::Asserted);
    assert_eq!(proposal.builds_on[0].as_str(), "assertion_overtime_v1");
    assert_eq!(serde_json::to_value(proposal).unwrap(), value);
}

#[test]
fn legacy_proposal_without_lineage_remains_compatible() {
    let proposal: ProposalCard = serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "proposal_legacy",
        "proposal_key": "legacy",
        "proposal_type": "question",
        "title": "Legacy proposal",
        "summary": "Predates provisional lineage.",
        "traceability": {
            "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
            "source_ids": [],
            "evidence_references": [],
            "supporting_claim_ids": []
        },
        "promotion_state": "proposed"
    }))
    .unwrap();

    assert!(proposal.builds_on.is_empty());
}
