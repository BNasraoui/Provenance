use super::initialized_store;
use crate::state_store::{CreatePromotionDecisionInput, CreateProposalCardInput, ProposalCard};
use provenance_core::{
    IdeationTarget, IdeationTargetType, IdentityType, PromotionActor, PromotionDecision,
    PromotionState, ProposalTraceability, ProposalType, ScopeId, StableId,
};

#[test]
fn proposal_state_requires_duplicate_or_superseded_link() {
    let (_dir, store, scope) = initialized_store();

    let err = store
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
                supporting_claim_ids: Vec::new(),
            },
            promotion_state: PromotionState::Duplicate,
            duplicate_of: None,
            superseded_by: None,
        })
        .unwrap_err();

    assert!(err
        .to_string()
        .contains("duplicate proposals must set duplicate_of"));
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
                supporting_claim_ids: Vec::new(),
            },
            promotion_state: PromotionState::Proposed,
            duplicate_of: None,
            superseded_by: None,
        })
        .unwrap();
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
    promotion_state: PromotionState,
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
            supporting_claim_ids: Vec::new(),
        },
        promotion_state,
        duplicate_of: None,
        superseded_by: None,
    }
}

#[test]
fn replacing_accepted_proposal_reports_human_disposition() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Original proposal",
            PromotionState::Accepted,
        ))
        .unwrap();

    let err = store
        .upsert_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Replacement proposal",
            PromotionState::Proposed,
        ))
        .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("human disposition"));
    assert!(message.contains("accepted"));
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
        .upsert_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Replacement proposal",
            PromotionState::Proposed,
        ))
        .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("human disposition"));
    assert!(message.contains("proposed"));
    assert!(message.contains("decision_overtime"));
}
