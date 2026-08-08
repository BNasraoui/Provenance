use crate::{
    layout::ProvenanceLayout,
    state_store::{
        CreateAssertionInput, CreateDispositionInput, CreateProposalCardInput, StateStore,
    },
};
use provenance_core::{
    AssertionId, DispositionActor, DispositionDecision, IdentityType, PromotionState,
    ProposalTraceability, ProposalType, ScopeId, StableId,
};

#[test]
fn modern_lifecycle_coexists_with_frozen_shipped_records() {
    let directory = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    let shipped = camino::Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(".provenance/state");
    copy_tree(&shipped, &layout.state_dir());
    let store = StateStore::new(layout);
    let scope = ScopeId::new("default").unwrap();
    let contribution: provenance_core::Contribution = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "contribution_modern",
        "target": {"artifact_type": "requirement", "artifact_id": "req_modern"},
        "participant_slot": "reviewer", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_modern", "evidence_type": "source", "summary": "Pinned"}],
        "material_claims": [{"claim_id": "claim_modern", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_modern"]}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    })).unwrap();
    let mut synthesis: provenance_core::SynthesisPacket = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "synthesis_modern",
        "target": {"artifact_type": "requirement", "artifact_id": "req_modern"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [],
        "evidence_gaps": [{"question": "Unverified", "needed_evidence_type": "source", "blocking_promotion": true}],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_id": "proposal_modern", "proposal_key": "modern", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["reviewer"]}],
        "required_human_decisions": []
    })).unwrap();
    let mut contributions = store.list_contributions(&scope).unwrap();
    contributions.push(contribution);
    contributions.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let contributions_path = crate::shards::contributions_path(&store.layout, &scope);
    write_jsonl(&contributions_path, &contributions);
    let mut synthesis_packets = store.list_synthesis_packets(&scope).unwrap();
    synthesis_packets.push(synthesis.clone());
    synthesis_packets.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let synthesis_path = crate::shards::synthesis_packets_path(&store.layout, &scope);
    write_jsonl(&synthesis_path, &synthesis_packets);
    store
        .create_proposal_card(CreateProposalCardInput {
            scope_id: scope.clone(),
            id: StableId::new("proposal_modern").unwrap(),
            proposal_key: "modern".into(),
            proposal_type: ProposalType::RequirementCandidate,
            title: "Modern".into(),
            summary: "Modern candidate".into(),
            confidence: None,
            traceability: ProposalTraceability {
                target: provenance_core::IdeationTarget {
                    artifact_type: provenance_core::IdeationTargetType::Requirement,
                    artifact_id: StableId::new("req_modern").unwrap(),
                },
                source_ids: vec![],
                evidence_references: vec![],
                supporting_claim_ids: vec![StableId::new("claim_modern").unwrap()],
            },
            promotion_state: PromotionState::Proposed,
            builds_on: vec![],
            duplicate_of: None,
            superseded_by: None,
        })
        .unwrap();
    synthesis.evidence_gaps.clear();
    let synthesis_id = synthesis.id.clone();
    *synthesis_packets
        .iter_mut()
        .find(|packet| packet.id == synthesis_id)
        .unwrap() = synthesis;
    write_jsonl(&synthesis_path, &synthesis_packets);
    store
        .assert_proposal(CreateAssertionInput {
            scope_id: scope.clone(),
            id: AssertionId::new("assertion_modern").unwrap(),
            proposal_id: StableId::new("proposal_modern").unwrap(),
            synthesis_packet_id: StableId::new("synthesis_modern").unwrap(),
            supporting_claim_ids: vec![StableId::new("claim_modern").unwrap()],
        })
        .unwrap();
    store
        .create_disposition(CreateDispositionInput {
            scope_id: scope,
            id: StableId::new("disposition_modern").unwrap(),
            proposal_id: StableId::new("proposal_modern").unwrap(),
            decision: DispositionDecision::Accepted,
            rationale: "Reviewed".into(),
            actor: DispositionActor {
                identity_type: IdentityType::Agent,
                id: "codex-review-panel-gpt55-medium".into(),
                name: None,
            },
            canonical_artifact: None,
            external_action: None,
        })
        .unwrap();
}

fn write_jsonl<T: serde::Serialize>(path: &camino::Utf8Path, records: &[T]) {
    crate::jsonl::write_jsonl_atomic(path, records).unwrap();
}

fn copy_tree(source: &camino::Utf8Path, destination: &camino::Utf8Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let child = camino::Utf8PathBuf::from_path_buf(entry.path()).unwrap();
        let target = destination.join(entry.file_name().to_string_lossy().as_ref());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&child, &target);
        } else {
            std::fs::copy(child, target).unwrap();
        }
    }
}
