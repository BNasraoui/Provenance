use super::*;
use provenance_core::{IdeationTargetType, UncertaintyLevel};

#[test]
fn ideation_output_records_are_written_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            ScopeId::new("default").unwrap(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    let store = StateStore::new(layout);
    let scope = ScopeId::new("default").unwrap();

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
