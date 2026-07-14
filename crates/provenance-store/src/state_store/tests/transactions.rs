use super::{initialized_store, seeded_requirement_store};
use crate::state_store::{
    CreatePromotionDecisionInput, CreateProposalCardInput, CreateResolutionInput,
};
use provenance_core::{
    IdeationTarget, IdeationTargetType, IdentityType, PromotionActor, PromotionDecision,
    PromotionState, ProposalTraceability, ProposalType, ResolutionStatus, SchemaVersion, Source,
    SourceType, StableId,
};
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn injected_io_failure_rolls_back_resolution_and_both_edges() {
    let (_dir, store, scope) = seeded_requirement_store();
    let failing = store.with_test_commit_failure(1);

    let error = failing
        .create_resolution(CreateResolutionInput {
            scope_id: scope.clone(),
            id: StableId::new("res_atomic").unwrap(),
            title: "Atomic".into(),
            requirement_id: Some(StableId::new("req_overtime").unwrap()),
            position: "all or nothing".into(),
            rationale: "consistency".into(),
            status: ResolutionStatus::Proposed,
            context: None,
            enforcement: None,
            confidence: None,
            inputs: Vec::new(),
            made_by: None,
            approved_by: None,
            approved_at: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("injected transaction I/O failure"));
    assert!(store.list_resolutions(&scope).unwrap().is_empty());
    assert!(store.list_edges().unwrap().is_empty());
}

#[test]
fn injected_io_failure_rolls_back_proposal_disposition() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(CreateProposalCardInput {
            scope_id: scope.clone(),
            id: StableId::new("proposal_atomic").unwrap(),
            proposal_key: "atomic".into(),
            proposal_type: ProposalType::RequirementCandidate,
            title: "Atomic disposition".into(),
            summary: "all or nothing".into(),
            confidence: None,
            traceability: ProposalTraceability {
                target: IdeationTarget {
                    artifact_type: IdeationTargetType::Source,
                    artifact_id: StableId::new("source_code").unwrap(),
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
    let failing = store.with_test_commit_failure(1);

    failing
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope.clone(),
            id: StableId::new("decision_atomic").unwrap(),
            proposal_id: StableId::new("proposal_atomic").unwrap(),
            decision: PromotionDecision::Accepted,
            rationale: "human review".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Human,
                id: "reviewer".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap_err();

    assert_eq!(
        store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
        PromotionState::Proposed
    );
    assert!(store.list_promotion_decisions(&scope).unwrap().is_empty());
}

#[test]
fn snapshots_wait_for_and_observe_one_complete_generation() {
    let (_dir, store, scope) = seeded_requirement_store();
    let (staged_tx, staged_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let writer = store.clone();
    let writer_scope = scope.clone();
    let writer_handle = std::thread::spawn(move || {
        writer
            .write_transaction(|transaction| {
                let requirement_path =
                    crate::shards::requirements_path(&writer.layout, &writer_scope);
                transaction.mutate_jsonl(
                    &requirement_path,
                    |requirements: &mut Vec<provenance_core::Requirement>| {
                        requirements[0].statement = "new generation".into();
                        Ok(())
                    },
                )?;
                transaction.replace_jsonl(
                    &crate::shards::sources_path(&writer.layout, &writer_scope),
                    &[Source {
                        schema_version: SchemaVersion(1),
                        scope_id: writer_scope.clone(),
                        id: StableId::new("source_new").unwrap(),
                        name: "new generation".into(),
                        source_type: SourceType::SystemState,
                        url: None,
                        reference: None,
                        commit_pin: None,
                        effective_date: None,
                        review_date: None,
                        superseded_by: None,
                        origin_thread: None,
                        origin_message: None,
                    }],
                )?;
                staged_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
            .unwrap();
    });
    staged_rx.recv().unwrap();

    let reader = store;
    let reader_scope = scope;
    let (snapshot_tx, snapshot_rx) = mpsc::channel();
    let reader_handle = std::thread::spawn(move || {
        snapshot_tx
            .send(reader.scope_snapshot(&reader_scope).unwrap())
            .unwrap();
    });
    assert!(snapshot_rx
        .recv_timeout(Duration::from_millis(100))
        .is_err());
    release_tx.send(()).unwrap();
    let snapshot = snapshot_rx.recv_timeout(Duration::from_secs(2)).unwrap();

    writer_handle.join().unwrap();
    reader_handle.join().unwrap();
    assert_eq!(snapshot.sources[0].id.as_str(), "source_new");
    assert_eq!(snapshot.requirements[0].statement, "new generation");
}
