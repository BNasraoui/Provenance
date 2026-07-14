use serde::{Deserialize, Serialize};

use super::{CanonicalArtifactType, IdentityType, PromotionDecision};
use crate::model::ids::{SchemaVersion, ScopeId, StableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionActor {
    #[serde(alias = "identityType")]
    pub identity_type: IdentityType,
    #[serde(alias = "userId")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalArtifact {
    #[serde(alias = "artifactType")]
    pub artifact_type: CanonicalArtifactType,
    #[serde(alias = "artifactId")]
    pub artifact_id: StableId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispositionRecord {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    #[serde(alias = "promotionDecisionId")]
    pub id: StableId,
    #[serde(alias = "proposalId")]
    pub proposal_id: StableId,
    pub decision: PromotionDecision,
    pub rationale: String,
    #[serde(alias = "decidedBy")]
    pub actor: PromotionActor,
    #[serde(
        default,
        alias = "canonicalArtifact",
        skip_serializing_if = "Option::is_none"
    )]
    pub canonical_artifact: Option<CanonicalArtifact>,
}
