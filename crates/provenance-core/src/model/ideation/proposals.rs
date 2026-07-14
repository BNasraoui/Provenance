use serde::{Deserialize, Serialize};

use super::{
    lifecycle::AssertionId, IdeationEvidenceReference, IdeationTarget, PromotionState, ProposalType,
};
use crate::model::ids::{SchemaVersion, ScopeId, StableId};
use crate::model::validation::deserialize_optional_confidence;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalTraceability {
    pub target: IdeationTarget,
    #[serde(alias = "sourceIds")]
    pub source_ids: Vec<StableId>,
    #[serde(alias = "evidenceReferences")]
    pub evidence_references: Vec<IdeationEvidenceReference>,
    #[serde(alias = "supportingClaimIds")]
    pub supporting_claim_ids: Vec<StableId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposalCard {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    #[serde(alias = "proposalId")]
    pub id: StableId,
    #[serde(alias = "proposalKey")]
    pub proposal_key: String,
    #[serde(alias = "proposalType")]
    pub proposal_type: ProposalType,
    pub title: String,
    pub summary: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_confidence",
        skip_serializing_if = "Option::is_none"
    )]
    pub confidence: Option<f64>,
    pub traceability: ProposalTraceability,
    #[serde(alias = "promotionState")]
    pub promotion_state: PromotionState,
    #[serde(default, alias = "buildsOn", skip_serializing_if = "Vec::is_empty")]
    pub builds_on: Vec<AssertionId>,
    #[serde(
        default,
        alias = "duplicateOfProposalId",
        skip_serializing_if = "Option::is_none"
    )]
    pub duplicate_of: Option<StableId>,
    #[serde(
        default,
        alias = "supersededByProposalId",
        skip_serializing_if = "Option::is_none"
    )]
    pub superseded_by: Option<StableId>,
}

/// Read model with lifecycle state derived from immutable assertion and disposition records.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProposalView {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub proposal_key: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    pub traceability: ProposalTraceability,
    pub promotion_state: PromotionState,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub builds_on: Vec<AssertionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_of: Option<StableId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<StableId>,
}

impl ProposalCard {
    pub fn project(&self, promotion_state: PromotionState) -> ProposalView {
        ProposalView {
            schema_version: self.schema_version,
            scope_id: self.scope_id.clone(),
            id: self.id.clone(),
            proposal_key: self.proposal_key.clone(),
            proposal_type: self.proposal_type,
            title: self.title.clone(),
            summary: self.summary.clone(),
            confidence: self.confidence,
            traceability: self.traceability.clone(),
            promotion_state,
            builds_on: self.builds_on.clone(),
            duplicate_of: self.duplicate_of.clone(),
            superseded_by: self.superseded_by.clone(),
        }
    }
}
