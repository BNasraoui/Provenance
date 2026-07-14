use serde::{Deserialize, Serialize};

use super::{
    ArtifactChangeType, ContributionStance, IdeationEvidenceReference, IdeationEvidenceType,
    IdeationTarget, IdeationTargetType, SpeculationMarker, UncertaintyLevel,
};
use crate::model::ids::{SchemaVersion, ScopeId, StableId};
use crate::model::validation::deserialize_optional_confidence;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialClaim {
    #[serde(alias = "claimId")]
    pub claim_id: StableId,
    pub statement: String,
    #[serde(alias = "evidenceType")]
    pub evidence_type: IdeationEvidenceType,
    #[serde(alias = "evidenceReferenceIds")]
    pub evidence_reference_ids: Vec<StableId>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_confidence",
        skip_serializing_if = "Option::is_none"
    )]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimChallenge {
    #[serde(alias = "claimId")]
    pub claim_id: StableId,
    pub objection: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedArtifactChange {
    #[serde(alias = "artifactType")]
    pub artifact_type: IdeationTargetType,
    #[serde(default, alias = "artifactId", skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<StableId>,
    #[serde(alias = "changeType")]
    pub change_type: ArtifactChangeType,
    #[serde(alias = "supportingClaimIds")]
    pub supporting_claim_ids: Vec<StableId>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedRecommendation {
    pub recommendation: String,
    pub marker: SpeculationMarker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncertaintyRating {
    pub level: UncertaintyLevel,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub target: IdeationTarget,
    #[serde(alias = "participantSlot")]
    pub participant_slot: String,
    pub stance: ContributionStance,
    #[serde(alias = "strongestFinding")]
    pub strongest_finding: String,
    #[serde(alias = "evidenceReferences")]
    pub evidence_references: Vec<IdeationEvidenceReference>,
    #[serde(alias = "materialClaims")]
    pub material_claims: Vec<MaterialClaim>,
    pub risks: Vec<String>,
    pub objections: Vec<String>,
    pub challenges: Vec<ClaimChallenge>,
    #[serde(alias = "suggestedArtifactChanges")]
    pub suggested_artifact_changes: Vec<SuggestedArtifactChange>,
    #[serde(alias = "unsupportedRecommendations")]
    pub unsupported_recommendations: Vec<UnsupportedRecommendation>,
    pub uncertainty: UncertaintyRating,
    #[serde(alias = "openQuestions")]
    pub open_questions: Vec<String>,
}
