use super::{initialized_store, proposals::proposal_input};
use crate::state_store::{IdeationLandingBatch, ProposalCard};
use provenance_core::{PromotionState, ScopeId};

#[test]
fn legacy_terminal_state_is_preserved_without_lifecycle_records() {
    let (_dir, store, scope) = initialized_store();
    let input = proposal_input(
        &scope,
        "proposal_legacy",
        "Legacy accepted proposal",
        PromotionState::Accepted,
    );
    let mut proposal = store.create_proposal_card(input).unwrap();
    proposal.promotion_state = PromotionState::Accepted;
    proposal.legacy_terminal = true;
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::proposal_cards_path(&store.layout, &scope),
        &[proposal],
    )
    .unwrap();

    assert_eq!(
        store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
        PromotionState::Accepted
    );
}

#[test]
fn unmarked_terminal_definition_is_not_legacy_compatibility() {
    let (_dir, store, scope) = initialized_store();
    let input = proposal_input(
        &scope,
        "proposal_forged",
        "Forged",
        PromotionState::Accepted,
    );
    let mut proposal = store.create_proposal_card(input).unwrap();
    proposal.promotion_state = PromotionState::Accepted;
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::proposal_cards_path(&store.layout, &scope),
        &[proposal],
    )
    .unwrap();

    let error = store
        .validate_ideation_scope(&scope)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("safely identifiable legacy terminal"),
        "{error}"
    );
}

#[test]
fn legacy_terminal_state_cannot_be_asserted() {
    let (_dir, store, scope) = initialized_store();
    let input = proposal_input(
        &scope,
        "proposal_legacy",
        "Legacy",
        PromotionState::Accepted,
    );
    let mut proposal = store.create_proposal_card(input).unwrap();
    proposal.promotion_state = PromotionState::Accepted;
    proposal.legacy_terminal = true;
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::proposal_cards_path(&store.layout, &scope),
        &[proposal],
    )
    .unwrap();

    let error = store
        .assert_proposal(crate::state_store::CreateAssertionInput {
            scope_id: scope,
            id: provenance_core::AssertionId::new("assertion_forged").unwrap(),
            proposal_id: provenance_core::StableId::new("proposal_legacy").unwrap(),
            synthesis_packet_id: provenance_core::StableId::new("synthesis_missing").unwrap(),
            supporting_claim_ids: vec![],
        })
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("legacy terminal proposal") && error.contains("frozen"),
        "{error}"
    );
}

#[test]
fn raw_duplicate_immutable_identity_is_rejected_before_overlay() {
    let (_dir, store, scope) = initialized_store();
    let proposal = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_duplicate",
            "Original",
            PromotionState::Proposed,
        ))
        .unwrap();
    let mut divergent = proposal;
    divergent.title = "Divergent".into();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::ideation_landings_path(&store.layout, &scope),
        &[IdeationLandingBatch {
            contributions: vec![],
            synthesis_packets: vec![],
            proposals: vec![divergent],
            assertions: vec![],
            dispositions: vec![],
        }],
    )
    .unwrap();

    let error = store
        .validate_ideation_scope(&scope)
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate immutable proposal"), "{error}");
}

#[test]
fn landing_batch_rejects_records_owned_by_another_scope() {
    let (_dir, store, scope) = initialized_store();
    let mut foreign = proposal_input(
        &ScopeId::new("other").unwrap(),
        "proposal_foreign",
        "Foreign proposal",
        PromotionState::Proposed,
    );
    foreign.proposal_key = "foreign".into();
    let proposal = ProposalCard {
        schema_version: provenance_core::SchemaVersion(1),
        scope_id: foreign.scope_id,
        id: foreign.id,
        proposal_key: foreign.proposal_key,
        proposal_type: foreign.proposal_type,
        title: foreign.title,
        summary: foreign.summary,
        confidence: foreign.confidence,
        traceability: foreign.traceability,
        promotion_state: PromotionState::Proposed,
        legacy_terminal: false,
        builds_on: foreign.builds_on,
        duplicate_of: foreign.duplicate_of,
        superseded_by: foreign.superseded_by,
    };

    let error = store
        .land_ideation_batch(
            &scope,
            IdeationLandingBatch {
                contributions: Vec::new(),
                synthesis_packets: Vec::new(),
                proposals: vec![proposal],
                assertions: Vec::new(),
                dispositions: Vec::new(),
            },
            false,
        )
        .unwrap_err();

    assert!(error.to_string().contains("must match landing scope"));
}
