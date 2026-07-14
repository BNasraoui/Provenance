use super::initialized_store;
use crate::state_store::{
    CreateAssertionInput, CreatePromotionDecisionInput, CreateProposalCardInput, ProposalCard,
};
use provenance_core::{
    AssertionId, Contribution, IdeationTarget, IdeationTargetType, IdentityType, PromotionActor,
    PromotionDecision, PromotionState, ProposalTraceability, ProposalType, ScopeId, StableId,
    SynthesisPacket,
};

#[test]
fn proposal_state_requires_duplicate_or_superseded_link() {
    let (_dir, store, scope) = initialized_store();

    let proposal = store
        .create_proposal_card(CreateProposalCardInput {
            scope_id: scope,
            id: StableId::new("proposal_duplicate").unwrap(),
            proposal_key: "duplicate".into(),
            proposal_type: ProposalType::RequirementCandidate,
            title: "Duplicate proposal".into(),
            summary: "This should point at the original proposal.".into(),
            confidence: None,
            traceability: ProposalTraceability {
                target: IdeationTarget {
                    artifact_type: IdeationTargetType::Requirement,
                    artifact_id: StableId::new("req_overtime").unwrap(),
                },
                source_ids: Vec::new(),
                evidence_references: Vec::new(),
                supporting_claim_ids: vec![StableId::new("claim_a").unwrap()],
            },
            builds_on: Vec::new(),
            duplicate_of: None,
            superseded_by: None,
        })
        .unwrap();

    assert_eq!(proposal.promotion_state, PromotionState::Proposed);
}

#[test]
fn promotion_decision_updates_proposal_state() {
    let (_dir, store, scope) = initialized_store();

    store
        .create_proposal_card(CreateProposalCardInput {
            scope_id: scope.clone(),
            id: StableId::new("proposal_overtime").unwrap(),
            proposal_key: "overtime".into(),
            proposal_type: ProposalType::RequirementCandidate,
            title: "Clarify overtime".into(),
            summary: "Clarify the overtime requirement.".into(),
            confidence: None,
            traceability: ProposalTraceability {
                target: IdeationTarget {
                    artifact_type: IdeationTargetType::Requirement,
                    artifact_id: StableId::new("req_overtime").unwrap(),
                },
                source_ids: Vec::new(),
                evidence_references: Vec::new(),
                supporting_claim_ids: vec![StableId::new("claim_a").unwrap()],
            },
            builds_on: Vec::new(),
            duplicate_of: None,
            superseded_by: None,
        })
        .unwrap();
    assert_existing(&store, &scope, "proposal_overtime");
    store
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope.clone(),
            id: StableId::new("decision_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: PromotionDecision::Accepted,
            rationale: "Approved by human review.".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap();

    assert_eq!(
        store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
        PromotionState::Accepted
    );
}

fn proposal_input(
    scope: &ScopeId,
    id: &str,
    title: &str,
    _promotion_state: PromotionState,
) -> CreateProposalCardInput {
    CreateProposalCardInput {
        scope_id: scope.clone(),
        id: StableId::new(id).unwrap(),
        proposal_key: "overtime".into(),
        proposal_type: ProposalType::RequirementCandidate,
        title: title.into(),
        summary: "Clarify the overtime requirement.".into(),
        confidence: None,
        traceability: ProposalTraceability {
            target: IdeationTarget {
                artifact_type: IdeationTargetType::Requirement,
                artifact_id: StableId::new("req_overtime").unwrap(),
            },
            source_ids: Vec::new(),
            evidence_references: Vec::new(),
            supporting_claim_ids: vec![StableId::new("claim_a").unwrap()],
        },
        builds_on: Vec::new(),
        duplicate_of: None,
        superseded_by: None,
    }
}

fn assert_existing(store: &crate::state_store::StateStore, scope: &ScopeId, proposal_id: &str) {
    let contribution: Contribution = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "contribution_a",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
        "participant_slot": "extractor", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_a", "evidence_type": "source", "summary": "Pinned"}],
        "material_claims": [{"claim_id": "claim_a", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_a"]}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    })).unwrap();
    let synthesis: SynthesisPacket = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "synthesis_a",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_key": "overtime", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["extractor"]}],
        "required_human_decisions": []
    })).unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::contributions_path(&store.layout, scope),
        &[contribution],
    )
    .unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::synthesis_packets_path(&store.layout, scope),
        &[synthesis],
    )
    .unwrap();
    store
        .assert_proposal(CreateAssertionInput {
            scope_id: scope.clone(),
            id: AssertionId::new(format!("assertion_{proposal_id}")).unwrap(),
            proposal_id: StableId::new(proposal_id).unwrap(),
            synthesis_packet_id: StableId::new("synthesis_a").unwrap(),
            supporting_claim_ids: vec![StableId::new("claim_a").unwrap()],
        })
        .unwrap();
}

#[test]
fn replacing_accepted_proposal_reports_human_disposition() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Original proposal",
            PromotionState::Proposed,
        ))
        .unwrap();
    assert_existing(&store, &scope, "proposal_overtime");
    store
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope.clone(),
            id: StableId::new("decision_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: PromotionDecision::Accepted,
            rationale: "Approved by human review.".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap();

    let err = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Replacement proposal",
            PromotionState::Proposed,
        ))
        .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("proposal already exists and is immutable"));
}

#[test]
fn replacing_proposed_proposal_with_decision_edge_reports_human_disposition() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Original proposal",
            PromotionState::Proposed,
        ))
        .unwrap();
    assert_existing(&store, &scope, "proposal_overtime");
    store
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope.clone(),
            id: StableId::new("decision_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: PromotionDecision::Accepted,
            rationale: "Approved by human review.".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap();
    let path = crate::shards::proposal_cards_path(&store.layout, &scope);
    store
        .mutate_jsonl_records(&path, |proposals: &mut Vec<ProposalCard>| {
            proposals[0].promotion_state = PromotionState::Proposed;
            Ok(())
        })
        .unwrap();

    let err = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Replacement proposal",
            PromotionState::Proposed,
        ))
        .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("proposal already exists and is immutable"));
}

#[test]
fn asserted_proposal_is_a_durable_base_for_a_derivative() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime_v1",
            "Evidence-backed behavior",
            PromotionState::Proposed,
        ))
        .unwrap();
    assert_existing(&store, &scope, "proposal_overtime_v1");
    let mut derivative = proposal_input(
        &scope,
        "proposal_overtime_v2",
        "Narrowed behavior",
        PromotionState::Proposed,
    );
    derivative.builds_on = vec![AssertionId::new("assertion_proposal_overtime_v1").unwrap()];

    let created = store.create_proposal_card(derivative).unwrap();
    assert_eq!(
        created.builds_on[0].as_str(),
        "assertion_proposal_overtime_v1"
    );

    let err = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime_v1",
            "Rewritten assertion",
            PromotionState::Proposed,
        ))
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("proposal already exists and is immutable"));
}

#[test]
fn proposal_lineage_must_reference_another_existing_proposal() {
    let (_dir, store, scope) = initialized_store();
    let mut input = proposal_input(
        &scope,
        "proposal_overtime_v2",
        "Narrowed behavior",
        PromotionState::Proposed,
    );
    input.builds_on = vec![AssertionId::new("assertion_missing").unwrap()];

    let err = store.create_proposal_card(input).unwrap_err();
    assert!(err
        .to_string()
        .contains("builds_on assertion assertion_missing does not exist"));
}

#[test]
fn behavior_changing_acceptance_requires_a_human_promotion_decision() {
    let (_dir, store, scope) = initialized_store();
    let created = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_direct_accept",
            "Direct acceptance",
            PromotionState::Accepted,
        ))
        .unwrap();
    assert_eq!(created.promotion_state, PromotionState::Proposed);

    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_asserted",
            "Evidence-backed behavior",
            PromotionState::Proposed,
        ))
        .unwrap();
    assert_existing(&store, &scope, "proposal_asserted");
    let err = store
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope,
            id: StableId::new("decision_agent_accept").unwrap(),
            proposal_id: StableId::new("proposal_asserted").unwrap(),
            decision: PromotionDecision::Accepted,
            rationale: "The swarm agreed.".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Agent,
                id: "swarm".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("authoritative disposition requires a human actor"));
}

#[test]
fn proposals_always_begin_proposed() {
    let (_dir, store, scope) = initialized_store();

    let created = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_new",
            "New proposal",
            PromotionState::Proposed,
        ))
        .unwrap();
    assert_eq!(created.promotion_state, PromotionState::Proposed);
}

#[test]
fn a_proposal_has_at_most_one_authoritative_disposition() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Overtime",
            PromotionState::Proposed,
        ))
        .unwrap();
    assert_existing(&store, &scope, "proposal_overtime");
    let decision = |id: &str, decision| CreatePromotionDecisionInput {
        scope_id: scope.clone(),
        id: StableId::new(id).unwrap(),
        proposal_id: StableId::new("proposal_overtime").unwrap(),
        decision,
        rationale: "Authoritative human disposition.".into(),
        actor: PromotionActor {
            identity_type: IdentityType::Human,
            id: "ben".into(),
            name: None,
        },
        canonical_artifact: None,
    };
    store
        .create_promotion_decision(decision("decision_accept", PromotionDecision::Accepted))
        .unwrap();

    let err = store
        .create_promotion_decision(decision("decision_reject", PromotionDecision::Rejected))
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("proposal already has an authoritative disposition"));
}

#[test]
fn assertion_is_a_verified_transition_and_lineage_targets_its_identity() {
    let (_dir, store, scope) = initialized_store();
    let mut parent = proposal_input(
        &scope,
        "proposal_parent",
        "Parent",
        PromotionState::Proposed,
    );
    parent.traceability.supporting_claim_ids = vec![StableId::new("claim_a").unwrap()];
    store.create_proposal_card(parent).unwrap();
    let contribution: Contribution = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "contribution_a",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
        "participant_slot": "extractor", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_a", "evidence_type": "source", "summary": "Pinned"}],
        "material_claims": [{"claim_id": "claim_a", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_a"]}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    })).unwrap();
    let synthesis: SynthesisPacket = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "synthesis_a",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_key": "overtime", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["extractor"]}],
        "required_human_decisions": []
    })).unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::contributions_path(&store.layout, &scope),
        &[contribution],
    )
    .unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::synthesis_packets_path(&store.layout, &scope),
        &[synthesis],
    )
    .unwrap();

    store
        .assert_proposal(CreateAssertionInput {
            scope_id: scope.clone(),
            id: AssertionId::new("assertion_parent").unwrap(),
            proposal_id: StableId::new("proposal_parent").unwrap(),
            synthesis_packet_id: StableId::new("synthesis_a").unwrap(),
            supporting_claim_ids: vec![StableId::new("claim_a").unwrap()],
        })
        .unwrap();
    assert_eq!(
        store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
        PromotionState::Asserted
    );

    let mut child = proposal_input(&scope, "proposal_child", "Child", PromotionState::Proposed);
    child.builds_on = vec![AssertionId::new("assertion_parent").unwrap()];
    store.create_proposal_card(child).unwrap();
}

#[test]
fn disposition_requires_a_prior_verified_assertion() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_unverified",
            "Unverified",
            PromotionState::Proposed,
        ))
        .unwrap();
    let err = store
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope,
            id: StableId::new("decision_unverified").unwrap(),
            proposal_id: StableId::new("proposal_unverified").unwrap(),
            decision: PromotionDecision::Rejected,
            rationale: "Not adjudicated.".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("must be asserted before disposition"));
}
