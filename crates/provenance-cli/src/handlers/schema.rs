use crate::{
    cli::{IdeationArtifactKind, SchemaCommand},
    output,
};
use provenance_core::{
    ArtifactChangeType, ContributionStance, EvidenceQuality, IdeationEvidenceType,
    IdeationTargetType, PromotionState, ProposalType, SpeculationMarker, UncertaintyLevel,
};
use serde::Serialize;
use serde_json::{json, Value};

pub(super) fn handle(command: SchemaCommand) -> anyhow::Result<()> {
    match command {
        SchemaCommand::Show { artifact, format } => {
            output::print(format, &schema_for(artifact))?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(super) fn schema_for(artifact: IdeationArtifactKind) -> Value {
    let schema = match artifact {
        IdeationArtifactKind::Contribution => json!({
            "title": "Contribution",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version",
                "scope_id",
                "id",
                "target",
                "participant_slot",
                "stance",
                "strongest_finding",
                "evidence_references",
                "material_claims",
                "risks",
                "objections",
                "challenges",
                "suggested_artifact_changes",
                "unsupported_recommendations",
                "uncertainty",
                "open_questions"
            ],
            "properties": {
                "schema_version": {"const": 1},
                "scope_id": {"$ref": "#/$defs/scopeId"},
                "id": {"$ref": "#/$defs/stableId"},
                "target": {"$ref": "#/$defs/ideationTarget"},
                "participant_slot": {"type": "string", "minLength": 1},
                "stance": {"enum": enum_names(&CONTRIBUTION_STANCES)},
                "strongest_finding": {"type": "string"},
                "evidence_references": {"type": "array", "items": {"$ref": "#/$defs/evidenceReference"}},
                "material_claims": {"type": "array", "items": {"$ref": "#/$defs/materialClaim"}},
                "risks": {"type": "array", "items": {"type": "string"}},
                "objections": {"type": "array", "items": {"type": "string"}},
                "challenges": {"type": "array", "items": {"$ref": "#/$defs/claimChallenge"}},
                "suggested_artifact_changes": {"type": "array", "items": {"$ref": "#/$defs/suggestedArtifactChange"}},
                "unsupported_recommendations": {"type": "array", "items": {"$ref": "#/$defs/unsupportedRecommendation"}},
                "uncertainty": {"$ref": "#/$defs/uncertainty"},
                "open_questions": {"type": "array", "items": {"type": "string"}}
            }
        }),
        IdeationArtifactKind::SynthesisPacket => json!({
            "title": "SynthesisPacket",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version",
                "scope_id",
                "id",
                "target",
                "summary",
                "consensus",
                "contested_claims",
                "minority_objections",
                "evidence_gaps",
                "unsupported_speculation",
                "open_questions",
                "suggested_artifacts",
                "required_human_decisions"
            ],
            "properties": {
                "schema_version": {"const": 1},
                "scope_id": {"$ref": "#/$defs/scopeId"},
                "id": {"$ref": "#/$defs/stableId"},
                "target": {"$ref": "#/$defs/ideationTarget"},
                "summary": {"type": "string"},
                "consensus": {"type": "array", "items": {"$ref": "#/$defs/consensusFinding"}},
                "contested_claims": {"type": "array", "items": {"$ref": "#/$defs/contestedClaim"}},
                "minority_objections": {"type": "array", "items": {"$ref": "#/$defs/minorityObjection"}},
                "evidence_gaps": {"type": "array", "items": {"$ref": "#/$defs/evidenceGap"}},
                "unsupported_speculation": {"type": "array", "items": {"$ref": "#/$defs/unsupportedSpeculation"}},
                "open_questions": {"type": "array", "items": {"type": "string"}},
                "suggested_artifacts": {"type": "array", "items": {"$ref": "#/$defs/suggestedArtifact"}},
                "required_human_decisions": {"type": "array", "items": {"$ref": "#/$defs/requiredHumanDecision"}}
            }
        }),
        IdeationArtifactKind::Proposal => json!({
            "title": "ProposalCard",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version",
                "scope_id",
                "id",
                "proposal_key",
                "proposal_type",
                "title",
                "summary",
                "traceability",
                "promotion_state"
            ],
            "properties": {
                "schema_version": {"const": 1},
                "scope_id": {"$ref": "#/$defs/scopeId"},
                "id": {"$ref": "#/$defs/stableId"},
                "proposal_key": {"type": "string", "minLength": 1},
                "proposal_type": {"enum": enum_names(&PROPOSAL_TYPES)},
                "title": {"type": "string"},
                "summary": {"type": "string"},
                "confidence": {"type": "number", "minimum": 0.0, "maximum": 1.0},
                "traceability": {"$ref": "#/$defs/proposalTraceability"},
                "promotion_state": {"enum": enum_names(&PROMOTION_STATES)},
                "duplicate_of": {"$ref": "#/$defs/stableId"},
                "superseded_by": {"$ref": "#/$defs/stableId"}
            }
        }),
    };

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "artifact": artifact.name(),
        "schema": schema,
        "$defs": common_defs()
    })
}

#[allow(clippy::too_many_lines)]
fn common_defs() -> Value {
    json!({
        "stableId": {
            "type": "string",
            "pattern": "^[a-z0-9_-]+$",
            "description": "StableId: lowercase ASCII letters, digits, '_' or '-'."
        },
        "scopeId": {
            "type": "string",
            "pattern": "^[a-z0-9_-]+$",
            "description": "ScopeId: lowercase ASCII letters, digits, '_' or '-'."
        },
        "stableIdArray": {
            "type": "array",
            "items": {"$ref": "#/$defs/stableId"}
        },
        "ideationTarget": {
            "type": "object",
            "additionalProperties": false,
            "required": ["artifact_type", "artifact_id"],
            "properties": {
                "artifact_type": {"enum": enum_names(&IDEATION_TARGET_TYPES)},
                "artifact_id": {"$ref": "#/$defs/stableId"}
            }
        },
        "evidenceReference": {
            "type": "object",
            "additionalProperties": false,
            "required": ["reference_id", "evidence_type", "summary"],
            "properties": {
                "reference_id": {"$ref": "#/$defs/stableId"},
                "evidence_type": {"enum": enum_names(&IDEATION_EVIDENCE_TYPES)},
                "summary": {"type": "string"},
                "file_path": {"type": "string"},
                "line": {"type": "integer", "minimum": 1}
            }
        },
        "materialClaim": {
            "type": "object",
            "additionalProperties": false,
            "required": ["claim_id", "statement", "evidence_type", "evidence_reference_ids"],
            "properties": {
                "claim_id": {"$ref": "#/$defs/stableId"},
                "statement": {"type": "string"},
                "evidence_type": {"enum": enum_names(&IDEATION_EVIDENCE_TYPES)},
                "evidence_reference_ids": {"$ref": "#/$defs/stableIdArray"},
                "confidence": {"type": "number", "minimum": 0.0, "maximum": 1.0}
            }
        },
        "claimChallenge": {
            "type": "object",
            "additionalProperties": false,
            "required": ["claim_id", "objection"],
            "properties": {
                "claim_id": {"$ref": "#/$defs/stableId"},
                "objection": {"type": "string"}
            }
        },
        "suggestedArtifactChange": {
            "type": "object",
            "additionalProperties": false,
            "required": ["artifact_type", "change_type", "supporting_claim_ids", "summary"],
            "properties": {
                "artifact_type": {"enum": enum_names(&IDEATION_TARGET_TYPES)},
                "artifact_id": {"$ref": "#/$defs/stableId"},
                "change_type": {"enum": enum_names(&ARTIFACT_CHANGE_TYPES)},
                "supporting_claim_ids": {"$ref": "#/$defs/stableIdArray"},
                "summary": {"type": "string"}
            }
        },
        "unsupportedRecommendation": {
            "type": "object",
            "additionalProperties": false,
            "required": ["recommendation", "marker"],
            "properties": {
                "recommendation": {"type": "string"},
                "marker": {"enum": enum_names(&SPECULATION_MARKERS)}
            }
        },
        "uncertainty": {
            "type": "object",
            "additionalProperties": false,
            "required": ["level", "rationale"],
            "properties": {
                "level": {"enum": enum_names(&UNCERTAINTY_LEVELS)},
                "rationale": {"type": "string"}
            }
        },
        "consensusFinding": {
            "type": "object",
            "additionalProperties": false,
            "required": ["statement", "supporting_participant_slots", "evidence_reference_ids"],
            "properties": {
                "statement": {"type": "string"},
                "supporting_participant_slots": {"type": "array", "items": {"type": "string"}},
                "evidence_reference_ids": {"$ref": "#/$defs/stableIdArray"}
            }
        },
        "contestedClaim": {
            "type": "object",
            "additionalProperties": false,
            "required": ["claim_id", "statement", "supporting_participant_slots", "opposing_participant_slots", "evidence_quality"],
            "properties": {
                "claim_id": {"$ref": "#/$defs/stableId"},
                "statement": {"type": "string"},
                "supporting_participant_slots": {"type": "array", "items": {"type": "string"}},
                "opposing_participant_slots": {"type": "array", "items": {"type": "string"}},
                "evidence_quality": {"enum": enum_names(&EVIDENCE_QUALITIES)}
            }
        },
        "minorityObjection": {
            "type": "object",
            "additionalProperties": false,
            "required": ["participant_slot", "objection", "evidence_reference_ids"],
            "properties": {
                "participant_slot": {"type": "string"},
                "objection": {"type": "string"},
                "evidence_reference_ids": {"$ref": "#/$defs/stableIdArray"}
            }
        },
        "evidenceGap": {
            "type": "object",
            "additionalProperties": false,
            "required": ["question", "needed_evidence_type", "blocking_promotion"],
            "properties": {
                "question": {"type": "string"},
                "needed_evidence_type": {"enum": enum_names(&IDEATION_EVIDENCE_TYPES)},
                "blocking_promotion": {"type": "boolean"}
            }
        },
        "unsupportedSpeculation": {
            "type": "object",
            "additionalProperties": false,
            "required": ["statement", "originating_participant_slots", "marker"],
            "properties": {
                "statement": {"type": "string"},
                "originating_participant_slots": {"type": "array", "items": {"type": "string"}},
                "marker": {"enum": enum_names(&SPECULATION_MARKERS)}
            }
        },
        "suggestedArtifact": {
            "type": "object",
            "additionalProperties": false,
            "required": ["proposal_key", "proposal_type", "summary", "origin_participant_slots"],
            "properties": {
                "proposal_key": {"type": "string", "minLength": 1},
                "proposal_type": {"enum": enum_names(&PROPOSAL_TYPES)},
                "summary": {"type": "string"},
                "origin_participant_slots": {"type": "array", "items": {"type": "string"}}
            }
        },
        "requiredHumanDecision": {
            "type": "object",
            "additionalProperties": false,
            "required": ["decision_key", "prompt", "blocks_promotion"],
            "properties": {
                "decision_key": {"$ref": "#/$defs/stableId"},
                "prompt": {"type": "string"},
                "blocks_promotion": {"type": "boolean"}
            }
        },
        "proposalTraceability": {
            "type": "object",
            "additionalProperties": false,
            "required": ["target", "source_ids", "evidence_references", "supporting_claim_ids"],
            "properties": {
                "target": {"$ref": "#/$defs/ideationTarget"},
                "source_ids": {"$ref": "#/$defs/stableIdArray"},
                "evidence_references": {"type": "array", "items": {"$ref": "#/$defs/evidenceReference"}},
                "supporting_claim_ids": {"$ref": "#/$defs/stableIdArray"}
            }
        }
    })
}

const IDEATION_TARGET_TYPES: [IdeationTargetType; 7] = [
    IdeationTargetType::Source,
    IdeationTargetType::Requirement,
    IdeationTargetType::Resolution,
    IdeationTargetType::Rule,
    IdeationTargetType::Topic,
    IdeationTargetType::Question,
    IdeationTargetType::Domain,
];

const IDEATION_EVIDENCE_TYPES: [IdeationEvidenceType; 6] = [
    IdeationEvidenceType::Source,
    IdeationEvidenceType::Artifact,
    IdeationEvidenceType::ThreadMessage,
    IdeationEvidenceType::DomainKnowledge,
    IdeationEvidenceType::Unsupported,
    IdeationEvidenceType::Exploratory,
];

const ARTIFACT_CHANGE_TYPES: [ArtifactChangeType; 4] = [
    ArtifactChangeType::Create,
    ArtifactChangeType::Update,
    ArtifactChangeType::Remove,
    ArtifactChangeType::None,
];

const CONTRIBUTION_STANCES: [ContributionStance; 4] = [
    ContributionStance::Support,
    ContributionStance::Oppose,
    ContributionStance::Mixed,
    ContributionStance::NeedsMoreEvidence,
];

const SPECULATION_MARKERS: [SpeculationMarker; 2] = [
    SpeculationMarker::Unsupported,
    SpeculationMarker::Exploratory,
];

const UNCERTAINTY_LEVELS: [UncertaintyLevel; 3] = [
    UncertaintyLevel::Low,
    UncertaintyLevel::Medium,
    UncertaintyLevel::High,
];

const EVIDENCE_QUALITIES: [EvidenceQuality; 4] = [
    EvidenceQuality::Strong,
    EvidenceQuality::Mixed,
    EvidenceQuality::Weak,
    EvidenceQuality::Unsupported,
];

const PROPOSAL_TYPES: [ProposalType; 6] = [
    ProposalType::RequirementCandidate,
    ProposalType::ResolutionCandidate,
    ProposalType::RuleCandidate,
    ProposalType::SourceGap,
    ProposalType::Question,
    ProposalType::NoAction,
];

const PROMOTION_STATES: [PromotionState; 6] = [
    PromotionState::Proposed,
    PromotionState::Accepted,
    PromotionState::Rejected,
    PromotionState::Deferred,
    PromotionState::Duplicate,
    PromotionState::Superseded,
];

fn enum_names<T: Serialize>(variants: &[T]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| {
            serde_json::to_value(variant)
                .expect("schema enum variant should serialize")
                .as_str()
                .expect("schema enum variant should serialize as a string")
                .to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let promotion_states = enum_names(&PROMOTION_STATES);

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
            enum_values_at(&proposal, "/schema/properties/promotion_state/enum"),
            promotion_states
        );
    }
}
