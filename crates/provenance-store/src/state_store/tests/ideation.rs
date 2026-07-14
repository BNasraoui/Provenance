use super::initialized_store;
use crate::state_store::{CreateContributionInput, IdeationLandingBatch};
use provenance_core::{
    ContributionStance, IdeationTarget, IdeationTargetType, StableId, UncertaintyLevel,
    UncertaintyRating,
};

fn contribution_input(scope: &provenance_core::ScopeId, finding: &str) -> CreateContributionInput {
    CreateContributionInput {
        scope_id: scope.clone(),
        id: StableId::new("contrib_visible").unwrap(),
        target: IdeationTarget {
            artifact_type: IdeationTargetType::Requirement,
            artifact_id: StableId::new("req_overtime").unwrap(),
        },
        participant_slot: "reviewer".into(),
        stance: ContributionStance::Support,
        strongest_finding: finding.into(),
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
    }
}

#[test]
fn direct_writer_cannot_hide_behind_a_landed_replacement() {
    let (_dir, store, scope) = initialized_store();
    let direct = store
        .create_contribution(contribution_input(&scope, "direct"))
        .unwrap();
    let mut landed = direct;
    landed.strongest_finding = "landed authority".into();
    store
        .land_ideation_batch(
            &scope,
            IdeationLandingBatch {
                contributions: vec![landed],
                synthesis_packets: vec![],
                proposals: vec![],
                assertions: vec![],
                dispositions: vec![],
            },
            true,
        )
        .unwrap();

    let error = store
        .upsert_contribution(contribution_input(&scope, "invisible update"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("landed contribution"), "{error}");
}

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
