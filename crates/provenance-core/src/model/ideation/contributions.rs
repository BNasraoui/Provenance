use serde::{Deserialize, Serialize};

use super::{
    ArtifactChangeType, ContributionStance, IdeationEvidenceReference, IdeationEvidenceType,
    IdeationTarget, IdeationTargetType, SpeculationMarker, UncertaintyLevel,
};
use crate::model::ids::{SchemaVersion, ScopeId, StableId};
use crate::model::validation::deserialize_optional_confidence;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialClaim {
    pub claim_id: StableId,
    pub statement: String,
    pub evidence_type: IdeationEvidenceType,
    pub evidence_reference_ids: Vec<StableId>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_confidence",
        skip_serializing_if = "Option::is_none"
    )]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimChallenge {
    pub claim_id: StableId,
    pub objection: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestedArtifactChange {
    pub artifact_type: IdeationTargetType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<StableId>,
    pub change_type: ArtifactChangeType,
    pub supporting_claim_ids: Vec<StableId>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedRecommendation {
    pub recommendation: String,
    pub marker: SpeculationMarker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UncertaintyRating {
    pub level: UncertaintyLevel,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contribution {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub target: IdeationTarget,
    pub participant_slot: String,
    pub stance: ContributionStance,
    pub strongest_finding: String,
    pub evidence_references: Vec<IdeationEvidenceReference>,
    pub material_claims: Vec<MaterialClaim>,
    pub risks: Vec<String>,
    pub objections: Vec<String>,
    pub challenges: Vec<ClaimChallenge>,
    pub suggested_artifact_changes: Vec<SuggestedArtifactChange>,
    pub unsupported_recommendations: Vec<UnsupportedRecommendation>,
    pub uncertainty: UncertaintyRating,
    pub open_questions: Vec<String>,
}
