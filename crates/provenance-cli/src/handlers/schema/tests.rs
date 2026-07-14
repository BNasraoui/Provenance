use super::{
    common::model::{
        enum_names, ARTIFACT_CHANGE_TYPES, CONTRIBUTION_STANCES, EVIDENCE_QUALITIES,
        IDEATION_EVIDENCE_TYPES, IDEATION_TARGET_TYPES, PROMOTION_STATES, PROPOSAL_TYPES,
        SPECULATION_MARKERS, UNCERTAINTY_LEVELS,
    },
    schema_for,
};
use crate::cli::IdeationArtifactKind;
use provenance_core::{
    ArtifactChangeType, ContributionStance, EvidenceQuality, IdeationEvidenceType,
    IdeationTargetType, PromotionState, ProposalType, SpeculationMarker, UncertaintyLevel,
};
use serde_json::Value;

fn enum_values_at(schema: &Value, pointer: &str) -> Vec<String> {
    schema
        .pointer(pointer)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

fn assert_ideation_target_type_array_is_exhaustive(value: IdeationTargetType) {
    match value {
        IdeationTargetType::Source
        | IdeationTargetType::Requirement
        | IdeationTargetType::Resolution
        | IdeationTargetType::Rule
        | IdeationTargetType::Topic
        | IdeationTargetType::Question
        | IdeationTargetType::Domain => {}
    }
}

fn assert_ideation_evidence_type_array_is_exhaustive(value: IdeationEvidenceType) {
    match value {
        IdeationEvidenceType::Source
        | IdeationEvidenceType::Artifact
        | IdeationEvidenceType::ThreadMessage
        | IdeationEvidenceType::DomainKnowledge
        | IdeationEvidenceType::Unsupported
        | IdeationEvidenceType::Exploratory => {}
    }
}

fn assert_artifact_change_type_array_is_exhaustive(value: ArtifactChangeType) {
    match value {
        ArtifactChangeType::Create
        | ArtifactChangeType::Update
        | ArtifactChangeType::Remove
        | ArtifactChangeType::None => {}
    }
}

fn assert_contribution_stance_array_is_exhaustive(value: ContributionStance) {
    match value {
        ContributionStance::Support
        | ContributionStance::Oppose
        | ContributionStance::Mixed
        | ContributionStance::NeedsMoreEvidence => {}
    }
}

fn assert_speculation_marker_array_is_exhaustive(value: SpeculationMarker) {
    match value {
        SpeculationMarker::Unsupported | SpeculationMarker::Exploratory => {}
    }
}

fn assert_uncertainty_level_array_is_exhaustive(value: UncertaintyLevel) {
    match value {
        UncertaintyLevel::Low | UncertaintyLevel::Medium | UncertaintyLevel::High => {}
    }
}

fn assert_evidence_quality_array_is_exhaustive(value: EvidenceQuality) {
    match value {
        EvidenceQuality::Strong
        | EvidenceQuality::Mixed
        | EvidenceQuality::Weak
        | EvidenceQuality::Unsupported => {}
    }
}

fn assert_proposal_type_array_is_exhaustive(value: ProposalType) {
    match value {
        ProposalType::RequirementCandidate
        | ProposalType::ResolutionCandidate
        | ProposalType::RuleCandidate
        | ProposalType::SourceGap
        | ProposalType::Question
        | ProposalType::NoAction => {}
    }
}

fn assert_promotion_state_array_is_exhaustive(value: PromotionState) {
    match value {
        PromotionState::Proposed
        | PromotionState::Asserted
        | PromotionState::Accepted
        | PromotionState::Rejected
        | PromotionState::Deferred
        | PromotionState::Duplicate
        | PromotionState::Superseded => {}
    }
}

#[test]
fn schema_enum_variant_arrays_are_exhaustive() {
    for variant in IDEATION_TARGET_TYPES {
        assert_ideation_target_type_array_is_exhaustive(variant);
    }
    for variant in IDEATION_EVIDENCE_TYPES {
        assert_ideation_evidence_type_array_is_exhaustive(variant);
    }
    for variant in ARTIFACT_CHANGE_TYPES {
        assert_artifact_change_type_array_is_exhaustive(variant);
    }
    for variant in CONTRIBUTION_STANCES {
        assert_contribution_stance_array_is_exhaustive(variant);
    }
    for variant in SPECULATION_MARKERS {
        assert_speculation_marker_array_is_exhaustive(variant);
    }
    for variant in UNCERTAINTY_LEVELS {
        assert_uncertainty_level_array_is_exhaustive(variant);
    }
    for variant in EVIDENCE_QUALITIES {
        assert_evidence_quality_array_is_exhaustive(variant);
    }
    for variant in PROPOSAL_TYPES {
        assert_proposal_type_array_is_exhaustive(variant);
    }
    for variant in PROMOTION_STATES {
        assert_promotion_state_array_is_exhaustive(variant);
    }
}

#[test]
fn schema_show_enum_values_match_model_serialization() {
    let contribution = schema_for(IdeationArtifactKind::Contribution);
    let synthesis = schema_for(IdeationArtifactKind::SynthesisPacket);
    let proposal = schema_for(IdeationArtifactKind::Proposal);
    let target_types = enum_names(&IDEATION_TARGET_TYPES);
    let evidence_types = enum_names(&IDEATION_EVIDENCE_TYPES);
    let change_types = enum_names(&ARTIFACT_CHANGE_TYPES);
    let contribution_stances = enum_names(&CONTRIBUTION_STANCES);
    let speculation_markers = enum_names(&SPECULATION_MARKERS);
    let uncertainty_levels = enum_names(&UNCERTAINTY_LEVELS);
    let evidence_qualities = enum_names(&EVIDENCE_QUALITIES);
    let proposal_types = enum_names(&PROPOSAL_TYPES);

    assert_eq!(
        enum_values_at(
            &contribution,
            "/$defs/ideationTarget/properties/artifact_type/enum"
        ),
        target_types
    );
    assert_eq!(
        enum_values_at(
            &contribution,
            "/$defs/evidenceReference/properties/evidence_type/enum"
        ),
        evidence_types
    );
    assert_eq!(
        enum_values_at(
            &contribution,
            "/$defs/materialClaim/properties/evidence_type/enum"
        ),
        evidence_types
    );
    assert_eq!(
        enum_values_at(
            &contribution,
            "/$defs/suggestedArtifactChange/properties/change_type/enum"
        ),
        change_types
    );
    assert_eq!(
        enum_values_at(&contribution, "/schema/properties/stance/enum"),
        contribution_stances
    );
    assert_eq!(
        enum_values_at(
            &contribution,
            "/$defs/unsupportedRecommendation/properties/marker/enum"
        ),
        speculation_markers
    );
    assert_eq!(
        enum_values_at(&contribution, "/$defs/uncertainty/properties/level/enum"),
        uncertainty_levels
    );
    assert_eq!(
        enum_values_at(
            &synthesis,
            "/$defs/evidenceGap/properties/needed_evidence_type/enum"
        ),
        evidence_types
    );
    assert_eq!(
        enum_values_at(
            &synthesis,
            "/$defs/contestedClaim/properties/evidence_quality/enum"
        ),
        evidence_qualities
    );
    assert_eq!(
        enum_values_at(
            &synthesis,
            "/$defs/unsupportedSpeculation/properties/marker/enum"
        ),
        speculation_markers
    );
    assert_eq!(
        enum_values_at(
            &synthesis,
            "/$defs/suggestedArtifact/properties/proposal_type/enum"
        ),
        proposal_types
    );
    assert_eq!(
        enum_values_at(&proposal, "/schema/properties/proposal_type/enum"),
        proposal_types
    );
    assert_eq!(
        proposal["schema"]["properties"]["promotion_state"]["const"],
        "proposed"
    );
}
