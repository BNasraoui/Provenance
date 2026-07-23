use super::initialized_store;
use crate::state_store::{
    CreateAssertionInput, CreateDispositionInput, CreateProposalCardInput, ProposalCard,
};
use provenance_core::{
    DispositionActor, DispositionDecision, IdeationTarget, IdeationTargetType, IdentityType,
    PromotionState, ProposalTraceability, ProposalType, ScopeId, StableId,
};

#[test]
fn legacy_disposition_path_reads_shipped_camel_case_records() {
    let (_dir, store, scope) = initialized_store();
    let path = crate::shards::legacy_promotion_decisions_path(&store.layout, &scope);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        r#"{"schema_version":1,"scope_id":"default","promotionDecisionId":"disposition_legacy","proposalId":"proposal_legacy","decision":"accepted","rationale":"Accepted.","decidedBy":{"identityType":"human","userId":"reviewer"},"canonicalArtifact":{"artifactType":"requirement","artifactId":"requirement_legacy"}}
"#,
    )
    .unwrap();

    let dispositions = store.list_dispositions(&scope).unwrap();
    assert_eq!(dispositions[0].id.as_str(), "disposition_legacy");
    assert_eq!(dispositions[0].actor.id, "reviewer");
}

#[test]
fn direct_modern_proposal_write_rejects_terminal_ingress() {
    let (_dir, store, scope) = initialized_store();
    let error = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_forged",
            "Forged terminal",
            PromotionState::Accepted,
        ))
        .unwrap_err()
        .to_string();

    assert!(error.contains("must begin proposed"), "{error}");
    assert!(store.list_proposal_cards(&scope).unwrap().is_empty());
}

#[test]
fn accepted_disposition_requires_an_assertion() {
    let (_dir, store, scope) = initialized_store();
    let contribution: provenance_core::Contribution = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "contribution_overtime",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
        "participant_slot": "reviewer", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_overtime", "evidence_type": "source", "summary": "Pinned"}],
        "material_claims": [{"claim_id": "claim_overtime", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_overtime"]}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    })).unwrap();
    let synthesis: provenance_core::SynthesisPacket = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "synthesis_overtime",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_id": "proposal_overtime", "proposal_key": "overtime", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["reviewer"]}],
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
    let mut proposal = proposal_input(
        &scope,
        "proposal_overtime",
        "Overtime",
        PromotionState::Proposed,
    );
    proposal.traceability.supporting_claim_ids = vec![StableId::new("claim_overtime").unwrap()];
    store.create_proposal_card(proposal).unwrap();

    let error = store
        .create_disposition(CreateDispositionInput {
            scope_id: scope,
            id: StableId::new("disposition_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: DispositionDecision::Accepted,
            rationale: "Reviewed".into(),
            actor: DispositionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap_err()
        .to_string();

    assert!(error.contains("must be asserted"), "{error}");
}

#[test]
fn rejected_disposition_does_not_require_an_assertion() {
    let (_dir, store, scope) = initialized_store();
    let mut manifest = store.manifest().unwrap();
    manifest.disposition_actor_ids.push("ben".into());
    std::fs::write(
        store.layout.manifest_path(),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_rejected",
            "Rejected",
            PromotionState::Proposed,
        ))
        .unwrap();

    store
        .create_disposition(CreateDispositionInput {
            scope_id: scope.clone(),
            id: StableId::new("disposition_rejected").unwrap(),
            proposal_id: StableId::new("proposal_rejected").unwrap(),
            decision: DispositionDecision::Rejected,
            rationale: "Did not pass adjudication".into(),
            actor: DispositionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap();

    assert_eq!(
        store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
        PromotionState::Rejected
    );
}

#[test]
fn direct_assertion_uses_the_aggregate_evidence_validator() {
    let (_dir, store, scope) = initialized_store();
    let contribution: provenance_core::Contribution = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "contribution_overtime",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
        "participant_slot": "reviewer", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_overtime", "evidence_type": "source", "summary": "Pinned"}],
        "material_claims": [{"claim_id": "claim_overtime", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_overtime"]}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    })).unwrap();
    let synthesis: provenance_core::SynthesisPacket = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "synthesis_overtime",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_id": "proposal_overtime", "proposal_key": "overtime", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["reviewer"]}],
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
    let mut proposal = proposal_input(
        &scope,
        "proposal_overtime",
        "Overtime",
        PromotionState::Proposed,
    );
    proposal.traceability.supporting_claim_ids = vec![StableId::new("claim_overtime").unwrap()];
    store.create_proposal_card(proposal).unwrap();

    let error = store
        .assert_proposal(CreateAssertionInput {
            scope_id: scope,
            id: provenance_core::AssertionId::new("assertion_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            synthesis_packet_id: StableId::new("synthesis_missing").unwrap(),
            supporting_claim_ids: vec![StableId::new("claim_overtime").unwrap()],
        })
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("synthesis packet") || error.contains("positive evidence"),
        "{error}"
    );
}

#[test]
fn repository_actor_allowlist_rejects_unlisted_disposition_actor() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Overtime",
            PromotionState::Proposed,
        ))
        .unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::assertion_records_path(&store.layout, &scope),
        &[provenance_core::AssertionRecord {
            schema_version: provenance_core::SchemaVersion(1),
            scope_id: scope.clone(),
            id: provenance_core::AssertionId::new("assertion_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            synthesis_packet_id: StableId::new("synthesis_overtime").unwrap(),
            supporting_claim_ids: vec![StableId::new("claim_overtime").unwrap()],
        }],
    )
    .unwrap();

    let error = store
        .create_disposition(CreateDispositionInput {
            scope_id: scope,
            id: StableId::new("disposition_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: DispositionDecision::Accepted,
            rationale: "Reviewed".into(),
            actor: DispositionActor {
                identity_type: IdentityType::Human,
                id: "forged-reviewer".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap_err()
        .to_string();

    assert!(error.contains("repository allowlist"), "{error}");
}

#[test]
fn proposal_projection_rejects_unlisted_disposition_actor() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Overtime",
            PromotionState::Proposed,
        ))
        .unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::dispositions_path(&store.layout, &scope),
        &[provenance_core::DispositionRecord {
            schema_version: provenance_core::SchemaVersion(1),
            scope_id: scope.clone(),
            id: StableId::new("disposition_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: DispositionDecision::Accepted,
            rationale: "Forged review".into(),
            actor: DispositionActor {
                identity_type: IdentityType::Human,
                id: "forged-reviewer".into(),
                name: None,
            },
            canonical_artifact: None,
        }],
    )
    .unwrap();

    let error = store.list_proposal_cards(&scope).unwrap_err().to_string();

    assert!(error.contains("repository allowlist"), "{error}");
}

#[test]
fn proposal_projection_uses_one_publication_snapshot() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Overtime",
            PromotionState::Proposed,
        ))
        .unwrap();
    let (validated_tx, validated_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let reader = {
        let store = store.clone();
        let scope = scope.clone();
        std::thread::spawn(move || {
            store.project_proposal_cards(&scope, || {
                validated_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
        })
    };
    validated_rx.recv().unwrap();

    let (published_tx, published_rx) = std::sync::mpsc::channel();
    let publisher = {
        let store = store.clone();
        let scope = scope.clone();
        std::thread::spawn(move || {
            store
                .with_repository_publication(|| {
                    crate::jsonl::write_jsonl_atomic::<ProposalCard>(
                        &crate::shards::proposal_cards_path(&store.layout, &scope),
                        &[],
                    )
                })
                .unwrap();
            published_tx.send(()).unwrap();
        })
    };

    assert!(published_rx
        .recv_timeout(std::time::Duration::from_millis(100))
        .is_err());
    release_tx.send(()).unwrap();
    let projected = reader.join().unwrap().unwrap();
    publisher.join().unwrap();

    assert_eq!(projected.len(), 1);
    assert!(store.list_proposal_cards(&scope).unwrap().is_empty());
}

#[test]
fn modern_proposal_is_immutable_even_when_upsert_is_requested() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Original",
            PromotionState::Proposed,
        ))
        .unwrap();

    let error = store
        .upsert_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Divergent",
            PromotionState::Proposed,
        ))
        .unwrap_err()
        .to_string();
    assert!(error.contains("immutable"), "{error}");
}

#[test]
fn disposition_derives_effective_state_without_mutating_proposal_definition() {
    let (_dir, store, scope) = initialized_store();
    let mut manifest = store.manifest().unwrap();
    manifest.disposition_actor_ids.push("ben".into());
    std::fs::write(
        store.layout.manifest_path(),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let contribution: provenance_core::Contribution = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "contribution_overtime",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
        "participant_slot": "reviewer", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_overtime", "evidence_type": "source", "summary": "Pinned"}],
        "material_claims": [{"claim_id": "claim_overtime", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_overtime"]}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    })).unwrap();
    let synthesis: provenance_core::SynthesisPacket = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "synthesis_overtime",
        "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_id": "proposal_overtime", "proposal_key": "overtime", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["reviewer"]}],
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
    let mut proposal = proposal_input(
        &scope,
        "proposal_overtime",
        "Overtime",
        PromotionState::Proposed,
    );
    proposal.traceability.supporting_claim_ids = vec![StableId::new("claim_overtime").unwrap()];
    store.create_proposal_card(proposal).unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::assertion_records_path(&store.layout, &scope),
        &[provenance_core::AssertionRecord {
            schema_version: provenance_core::SchemaVersion(1),
            scope_id: scope.clone(),
            id: provenance_core::AssertionId::new("assertion_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            synthesis_packet_id: StableId::new("synthesis_overtime").unwrap(),
            supporting_claim_ids: vec![StableId::new("claim_overtime").unwrap()],
        }],
    )
    .unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut threads = Vec::new();
    for id in ["disposition_overtime_a", "disposition_overtime_b"] {
        let store = store.clone();
        let scope = scope.clone();
        let barrier = barrier.clone();
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            store.create_disposition(CreateDispositionInput {
                scope_id: scope,
                id: StableId::new(id).unwrap(),
                proposal_id: StableId::new("proposal_overtime").unwrap(),
                decision: DispositionDecision::Accepted,
                rationale: "Reviewed".into(),
                actor: DispositionActor {
                    identity_type: IdentityType::Human,
                    id: "ben".into(),
                    name: None,
                },
                canonical_artifact: None,
            })
        }));
    }
    barrier.wait();
    let results = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(store.list_dispositions(&scope).unwrap().len(), 1);

    let persisted: Vec<ProposalCard> = serde_json::from_str(&format!(
        "[{}]",
        std::fs::read_to_string(crate::shards::proposal_cards_path(&store.layout, &scope))
            .unwrap()
            .trim()
    ))
    .unwrap();
    assert_eq!(persisted[0].promotion_state, PromotionState::Proposed);
    assert_eq!(
        store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
        PromotionState::Accepted
    );
}

#[test]
fn validation_rejects_divergent_duplicate_before_landing_overlay() {
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
    divergent.title = "Forged overlay".into();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::ideation_landings_path(&store.layout, &scope),
        &[crate::state_store::IdeationLandingBatch {
            contributions: Vec::new(),
            synthesis_packets: Vec::new(),
            proposals: vec![divergent],
            assertions: Vec::new(),
            dispositions: Vec::new(),
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
fn unregistered_terminal_row_is_not_legacy_compatibility() {
    let (_dir, store, scope) = initialized_store();
    let mut proposal = store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_forged",
            "Forged",
            PromotionState::Proposed,
        ))
        .unwrap();
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
    assert!(error.contains("frozen shipped-v1 fingerprint"), "{error}");
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
        builds_on: Vec::new(),
        promotion_state,
        duplicate_of: None,
        superseded_by: None,
    }
}
