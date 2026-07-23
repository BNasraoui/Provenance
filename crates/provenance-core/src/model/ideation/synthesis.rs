use serde::{Deserialize, Serialize};

use super::{
    EvidenceQuality, IdeationEvidenceType, IdeationTarget, ProposalType, SpeculationMarker,
};
use crate::model::ids::{SchemaVersion, ScopeId, StableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusFinding {
    pub statement: String,
    pub supporting_participant_slots: Vec<String>,
    pub evidence_reference_ids: Vec<StableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContestedClaim {
    pub claim_id: StableId,
    pub statement: String,
    pub supporting_participant_slots: Vec<String>,
    pub opposing_participant_slots: Vec<String>,
    pub evidence_quality: EvidenceQuality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinorityObjection {
    pub participant_slot: String,
    pub objection: String,
    pub evidence_reference_ids: Vec<StableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceGap {
    pub question: String,
    pub needed_evidence_type: IdeationEvidenceType,
    pub blocking_promotion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedSpeculation {
    pub statement: String,
    pub originating_participant_slots: Vec<String>,
    pub marker: SpeculationMarker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestedArtifact {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<StableId>,
    pub proposal_key: String,
    pub proposal_type: ProposalType,
    pub summary: String,
    pub origin_participant_slots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredHumanDecision {
    pub decision_key: StableId,
    pub prompt: String,
    pub blocks_promotion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynthesisPacket {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub target: IdeationTarget,
    pub summary: String,
    pub consensus: Vec<ConsensusFinding>,
    pub contested_claims: Vec<ContestedClaim>,
    pub minority_objections: Vec<MinorityObjection>,
    pub evidence_gaps: Vec<EvidenceGap>,
    pub unsupported_speculation: Vec<UnsupportedSpeculation>,
    pub open_questions: Vec<String>,
    pub suggested_artifacts: Vec<SuggestedArtifact>,
    pub required_human_decisions: Vec<RequiredHumanDecision>,
}
