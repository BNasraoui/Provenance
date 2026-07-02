use camino::Utf8PathBuf;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use super::enums::{
    ArtifactChangeType, ArtifactLinkTargetType, CanonicalArtifactType, ContributionStance,
    EdgeType, EvidenceQuality, IdeationEvidenceType, IdeationTargetType, IdentityType, MessageRole,
    NodeType, PromotionDecision, PromotionState, ProposalType, QuestionStatus, RequirementStatus,
    ResolutionInputType, ResolutionMethod, ResolutionStatus, RuleModality, RuleSeverity,
    RuleStatus, RuleType, ServiceBindingType, ServiceEnvironment, ServiceStatus, ServiceTier,
    SourceType, SpeculationMarker, ThreadStatus, TopicStatus, UncertaintyLevel,
};
use super::ids::{SchemaVersion, ScopeId, StableId};

fn empty_object() -> serde_json::Value {
    serde_json::json!({})
}

const fn empty_array() -> serde_json::Value {
    serde_json::json!([])
}

pub fn validate_confidence_score(confidence: f64) -> anyhow::Result<()> {
    anyhow::ensure!(
        confidence.is_finite() && (0.0..=1.0).contains(&confidence),
        "confidence must be between 0.0 and 1.0"
    );
    Ok(())
}

pub fn validate_optional_confidence_score(confidence: Option<f64>) -> anyhow::Result<Option<f64>> {
    match confidence {
        Some(confidence) => {
            validate_confidence_score(confidence)?;
            Ok(Some(confidence))
        }
        None => Ok(None),
    }
}

fn deserialize_optional_confidence<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let confidence = Option::<f64>::deserialize(deserializer)?;
    validate_optional_confidence_score(confidence).map_err(D::Error::custom)
}

pub fn validate_commit_pin(commit_pin: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        (7..=64).contains(&commit_pin.len())
            && commit_pin.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "commit pin must be a 7-64 character hexadecimal Git commit"
    );
    Ok(())
}

pub fn validate_optional_commit_pin(commit_pin: Option<String>) -> anyhow::Result<Option<String>> {
    match commit_pin {
        Some(commit_pin) => {
            validate_commit_pin(&commit_pin)?;
            Ok(Some(commit_pin))
        }
        None => Ok(None),
    }
}

fn deserialize_optional_commit_pin<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let commit_pin = Option::<String>::deserialize(deserializer)?;
    validate_optional_commit_pin(commit_pin).map_err(D::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    #[serde(alias = "sourceType")]
    pub source_type: SourceType,
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(
        default,
        alias = "commitPin",
        deserialize_with = "deserialize_optional_commit_pin",
        skip_serializing_if = "Option::is_none"
    )]
    pub commit_pin: Option<String>,
    #[serde(
        default,
        alias = "effectiveDate",
        skip_serializing_if = "Option::is_none"
    )]
    pub effective_date: Option<i64>,
    #[serde(default, alias = "reviewDate", skip_serializing_if = "Option::is_none")]
    pub review_date: Option<i64>,
    #[serde(
        default,
        alias = "supersededBy",
        skip_serializing_if = "Option::is_none"
    )]
    pub superseded_by: Option<StableId>,
    #[serde(
        default,
        alias = "originThread",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_thread: Option<StableId>,
    #[serde(
        default,
        alias = "originMessage",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_message: Option<StableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReference {
    #[serde(alias = "sourceId")]
    pub source_id: StableId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLink {
    #[serde(alias = "targetType")]
    pub target_type: ArtifactLinkTargetType,
    #[serde(alias = "targetId")]
    pub target_id: StableId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Deliberately unstructured free text: the dim view of decisions and
    /// investigations that are coming but cannot yet be phrased sharply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog: Option<String>,
    pub status: RequirementStatus,
    #[serde(default, alias = "domainId", skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<StableId>,
    #[serde(default, alias = "sourceRefs", skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<SourceReference>,
    #[serde(
        default,
        alias = "originThread",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_thread: Option<StableId>,
    #[serde(
        default,
        alias = "originMessage",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_message: Option<StableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boundary {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "requirementId")]
    pub requirement_id: StableId,
    pub statement: String,
    #[serde(default, alias = "sourceRef", skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<SourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "requirementId")]
    pub requirement_id: StableId,
    pub title: String,
    pub status: TopicStatus,
    #[serde(default, alias = "claimedBy", skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    #[serde(default, alias = "claimedAt", skip_serializing_if = "Option::is_none")]
    pub claimed_at: Option<i64>,
    #[serde(default)]
    pub links: Vec<ArtifactLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "topicId")]
    pub topic_id: StableId,
    #[serde(alias = "requirementId")]
    pub requirement_id: StableId,
    pub question: String,
    /// The verb that resolves this question, chosen when the question is minted.
    #[serde(alias = "resolutionMethod")]
    pub resolution_method: ResolutionMethod,
    pub status: QuestionStatus,
    #[serde(default, alias = "claimedBy", skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    #[serde(default, alias = "claimedAt", skip_serializing_if = "Option::is_none")]
    pub claimed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(default)]
    pub links: Vec<ArtifactLink>,
    #[serde(
        default,
        alias = "resolutionId",
        skip_serializing_if = "Option::is_none"
    )]
    pub resolution_id: Option<StableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionInput {
    #[serde(alias = "inputType")]
    pub input_type: ResolutionInputType,
    pub reference: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resolution {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub title: String,
    pub position: String,
    pub rationale: String,
    pub status: ResolutionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_confidence",
        skip_serializing_if = "Option::is_none"
    )]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub inputs: Vec<ResolutionInput>,
    #[serde(default, alias = "madeBy", skip_serializing_if = "Option::is_none")]
    pub made_by: Option<String>,
    #[serde(default, alias = "approvedBy", skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(default, alias = "approvedAt", skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<i64>,
    #[serde(
        default,
        alias = "supersededBy",
        skip_serializing_if = "Option::is_none"
    )]
    pub superseded_by: Option<StableId>,
    #[serde(alias = "reviewOn")]
    pub review_on: Option<String>,
    #[serde(default = "empty_array", alias = "reviewTriggers")]
    pub review_triggers: serde_json::Value,
    #[serde(
        default,
        alias = "originThread",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_thread: Option<StableId>,
    #[serde(
        default,
        alias = "originMessage",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_message: Option<StableId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "ruleCode")]
    pub rule_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub statement: String,
    pub status: RuleStatus,
    pub severity: RuleSeverity,
    #[serde(default, alias = "ruleType", skip_serializing_if = "Option::is_none")]
    pub rule_type: Option<RuleType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modality: Option<RuleModality>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_confidence",
        skip_serializing_if = "Option::is_none"
    )]
    pub confidence: Option<f64>,
    #[serde(
        default,
        alias = "extractionMethod",
        skip_serializing_if = "Option::is_none"
    )]
    pub extraction_method: Option<String>,
    #[serde(
        default,
        alias = "sourceDocument",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_document: Option<String>,
    #[serde(
        default,
        alias = "sourceSection",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_section: Option<String>,
    #[serde(
        default,
        alias = "originThread",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_thread: Option<StableId>,
    #[serde(
        default,
        alias = "originMessage",
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_message: Option<StableId>,
    #[serde(default = "empty_object")]
    pub expression: serde_json::Value,
    #[serde(default = "empty_array")]
    pub inputs: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ServiceEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<ServiceTier>,
    #[serde(default, alias = "externalId", skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub status: ServiceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceBinding {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "ruleId")]
    pub rule_id: StableId,
    #[serde(alias = "serviceId")]
    pub service_id: StableId,
    #[serde(alias = "bindingType")]
    pub binding_type: ServiceBindingType,
}

impl ServiceBinding {
    pub fn stable_id(
        rule_id: &StableId,
        service_id: &StableId,
        binding_type: ServiceBindingType,
    ) -> anyhow::Result<StableId> {
        let binding_type = match binding_type {
            ServiceBindingType::Enforces => "enforces",
            ServiceBindingType::Consumes => "consumes",
            ServiceBindingType::Monitors => "monitors",
        };
        StableId::new(format!(
            "service_binding_{}_{}_{}",
            rule_id.as_str(),
            service_id.as_str(),
            binding_type,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub edge_type: EdgeType,
    pub from_type: NodeType,
    pub from_id: StableId,
    pub to_type: NodeType,
    pub to_id: StableId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadParent {
    pub node_type: NodeType,
    pub node_id: StableId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thread {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub parent: ThreadParent,
    pub status: ThreadStatus,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub thread_id: StableId,
    pub role: MessageRole,
    #[serde(alias = "content")]
    pub body: String,
    pub created_at: i64,
    #[serde(default, alias = "aiMetadata", skip_serializing_if = "Option::is_none")]
    pub ai_metadata: Option<serde_json::Value>,
}

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

impl Edge {
    pub fn stable_id(
        edge_type: EdgeType,
        from_type: NodeType,
        from_id: &StableId,
        to_type: NodeType,
        to_id: &StableId,
    ) -> anyhow::Result<StableId> {
        StableId::new(
            format!(
                "{:?}_{:?}_{}_to_{:?}_{}",
                edge_type,
                from_type,
                from_id.as_str(),
                to_type,
                to_id.as_str()
            )
            .to_ascii_lowercase(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepoPathPrefix(Utf8PathBuf);

impl RepoPathPrefix {
    pub fn new(value: impl Into<Utf8PathBuf>) -> Self {
        Self(value.into())
    }
    pub fn as_path(&self) -> &camino::Utf8Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub id: ScopeId,
    pub path_prefix: RepoPathPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: SchemaVersion,
    pub scopes: Vec<Scope>,
}

impl Manifest {
    pub fn default_with_scope(scope: ScopeId, path_prefix: RepoPathPrefix) -> Self {
        Self {
            schema_version: SchemaVersion(1),
            scopes: vec![Scope {
                id: scope,
                path_prefix,
            }],
        }
    }
}
