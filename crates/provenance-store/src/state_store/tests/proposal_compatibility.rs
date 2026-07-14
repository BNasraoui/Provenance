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
