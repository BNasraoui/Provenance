use serde::{Deserialize, Serialize};

use super::validation::deserialize_optional_confidence;
use crate::model::{
    ArtifactChangeType, CanonicalArtifactType, ContributionStance, EvidenceQuality,
    IdeationEvidenceType, IdeationTargetType, IdentityType, PromotionDecision, PromotionState,
    ProposalType, SchemaVersion, ScopeId, SpeculationMarker, StableId, UncertaintyLevel,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeationTarget {
    #[serde(alias = "artifactType")]
    pub artifact_type: IdeationTargetType,
    #[serde(alias = "artifactId")]
    pub artifact_id: StableId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeationEvidenceReference {
    #[serde(alias = "referenceId")]
    pub reference_id: StableId,
    #[serde(alias = "evidenceType")]
    pub evidence_type: IdeationEvidenceType,
    pub summary: String,
    #[serde(default, alias = "filePath", skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusFinding {
    pub statement: String,
    #[serde(alias = "supportingParticipantSlots")]
    pub supporting_participant_slots: Vec<String>,
    #[serde(alias = "evidenceReferenceIds")]
    pub evidence_reference_ids: Vec<StableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContestedClaim {
    #[serde(alias = "claimId")]
    pub claim_id: StableId,
    pub statement: String,
    #[serde(alias = "supportingParticipantSlots")]
    pub supporting_participant_slots: Vec<String>,
    #[serde(alias = "opposingParticipantSlots")]
    pub opposing_participant_slots: Vec<String>,
    #[serde(alias = "evidenceQuality")]
    pub evidence_quality: EvidenceQuality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinorityObjection {
    #[serde(alias = "participantSlot")]
    pub participant_slot: String,
    pub objection: String,
    #[serde(alias = "evidenceReferenceIds")]
    pub evidence_reference_ids: Vec<StableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceGap {
    pub question: String,
    #[serde(alias = "neededEvidenceType")]
    pub needed_evidence_type: IdeationEvidenceType,
    #[serde(alias = "blockingPromotion")]
    pub blocking_promotion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedSpeculation {
    pub statement: String,
    #[serde(alias = "originatingParticipantSlots")]
    pub originating_participant_slots: Vec<String>,
    pub marker: SpeculationMarker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedArtifact {
    #[serde(alias = "proposalKey")]
    pub proposal_key: String,
    #[serde(alias = "proposalType")]
    pub proposal_type: ProposalType,
    pub summary: String,
    #[serde(alias = "originParticipantSlots")]
    pub origin_participant_slots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredHumanDecision {
    #[serde(alias = "decisionKey")]
    pub decision_key: StableId,
    pub prompt: String,
    #[serde(alias = "blocksPromotion")]
    pub blocks_promotion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynthesisPacket {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub target: IdeationTarget,
    pub summary: String,
    pub consensus: Vec<ConsensusFinding>,
    #[serde(alias = "contestedClaims")]
    pub contested_claims: Vec<ContestedClaim>,
    #[serde(alias = "minorityObjections")]
    pub minority_objections: Vec<MinorityObjection>,
    #[serde(alias = "evidenceGaps")]
    pub evidence_gaps: Vec<EvidenceGap>,
    #[serde(alias = "unsupportedSpeculation")]
    pub unsupported_speculation: Vec<UnsupportedSpeculation>,
    #[serde(alias = "openQuestions")]
    pub open_questions: Vec<String>,
    #[serde(alias = "suggestedArtifacts")]
    pub suggested_artifacts: Vec<SuggestedArtifact>,
    #[serde(alias = "requiredHumanDecisions")]
    pub required_human_decisions: Vec<RequiredHumanDecision>,
}

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
pub struct PromotionDecisionRecord {
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
