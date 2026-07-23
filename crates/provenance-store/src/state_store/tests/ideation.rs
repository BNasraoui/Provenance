use super::initialized_store;
use crate::state_store::CreateContributionInput;
use provenance_core::{
    ContributionStance, IdeationTarget, IdeationTargetType, StableId, UncertaintyLevel,
    UncertaintyRating,
};

#[test]
fn ideation_output_records_are_written_deterministically() {
    let (_dir, store, scope) = initialized_store();

    store
        .create_contribution(CreateContributionInput {
            scope_id: scope.clone(),
            id: StableId::new("contrib_b").unwrap(),
            target: IdeationTarget {
                artifact_type: IdeationTargetType::Requirement,
                artifact_id: StableId::new("req_overtime").unwrap(),
            },
            participant_slot: "reviewer".into(),
            stance: ContributionStance::Support,
            strongest_finding: "Supported by evidence".into(),
            evidence_references: Vec::new(),
            material_claims: Vec::new(),
            risks: Vec::new(),
            objections: Vec::new(),
            challenges: Vec::new(),
            suggested_artifact_changes: Vec::new(),
            unsupported_recommendations: Vec::new(),
            uncertainty: UncertaintyRating {
                level: UncertaintyLevel::Low,
                rationale: "Direct evidence".into(),
            },
            open_questions: Vec::new(),
        })
        .unwrap();
    store
        .create_contribution(CreateContributionInput {
            scope_id: scope.clone(),
            id: StableId::new("contrib_a").unwrap(),
            target: IdeationTarget {
                artifact_type: IdeationTargetType::Requirement,
                artifact_id: StableId::new("req_overtime").unwrap(),
            },
            participant_slot: "refuter".into(),
            stance: ContributionStance::NeedsMoreEvidence,
            strongest_finding: "Needs more evidence".into(),
            evidence_references: Vec::new(),
            material_claims: Vec::new(),
            risks: Vec::new(),
            objections: Vec::new(),
            challenges: Vec::new(),
            suggested_artifact_changes: Vec::new(),
            unsupported_recommendations: Vec::new(),
            uncertainty: UncertaintyRating {
                level: UncertaintyLevel::High,
                rationale: "Missing source".into(),
            },
            open_questions: Vec::new(),
        })
        .unwrap();

    assert_eq!(
        store.list_contributions(&scope).unwrap()[0].id.as_str(),
        "contrib_a"
    );
}

#[test]
fn invalid_lifecycle_batch_is_rejected_without_partial_writes() {
    let (_dir, store, scope) = initialized_store();
    let batch: crate::state_store::IdeationLandingBatch =
        serde_json::from_value(serde_json::json!({
            "contributions": [{
                "schema_version": 1, "scope_id": "default", "id": "contribution_a",
                "target": {"artifact_type": "requirement", "artifact_id": "req_a"},
                "participant_slot": "extractor", "stance": "support", "strongest_finding": "Observed",
                "evidence_references": [], "material_claims": [], "risks": [], "objections": [],
                "challenges": [], "suggested_artifact_changes": [], "unsupported_recommendations": [],
                "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
            }],
            "synthesis_packets": [],
            "proposals": [],
            "assertions": [{
                "schema_version": 1, "scope_id": "default", "id": "assertion_bad",
                "proposal_id": "proposal_missing", "synthesis_packet_id": "synthesis_missing",
                "supporting_claim_ids": []
            }],
            "dispositions": []
        }))
        .unwrap();

    store.land_ideation_batch(&scope, batch, false).unwrap_err();
    assert!(store.list_contributions(&scope).unwrap().is_empty());
    assert!(store.list_assertion_records(&scope).unwrap().is_empty());
}
