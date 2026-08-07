use serde::{Deserialize, Serialize};

use super::{CanonicalArtifactType, DispositionDecision, IdentityType};
use crate::model::ids::{SchemaVersion, ScopeId, StableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionActor {
    pub identity_type: IdentityType,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalArtifact {
    pub artifact_type: CanonicalArtifactType,
    pub artifact_id: StableId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionRecord {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub proposal_id: StableId,
    pub decision: DispositionDecision,
    pub rationale: String,
    pub actor: DispositionActor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_artifact: Option<CanonicalArtifact>,
}

pub fn validate_disposition_intrinsic(disposition: &DispositionRecord) -> anyhow::Result<()> {
    anyhow::ensure!(
        !disposition.rationale.trim().is_empty(),
        "disposition rationale must not be empty"
    );
    anyhow::ensure!(
        !disposition.actor.id.trim().is_empty(),
        "disposition actor id must not be empty"
    );
    Ok(())
}
