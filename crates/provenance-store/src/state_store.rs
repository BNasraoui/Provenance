mod domain_service_writers;
mod ideation_batches;
mod ideation_readers;
mod ideation_writers;
mod rule_writers;
mod shaping_writers;
mod thread_writers;
mod writers;

use crate::{layout::ProvenanceLayout, shards};
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_core::{
    ArtifactLink, AssertionId, AssertionRecord, Boundary, CanonicalArtifact, ClaimChallenge,
    ConsensusFinding, ContestedClaim, Contribution, ContributionStance, DispositionRecord, Domain,
    Edge, EdgeType, EvidenceGap, IdeationEvidenceReference, IdeationTarget, Manifest,
    MaterialClaim, Message, MessageRole, MinorityObjection, NodeType, PromotionActor,
    PromotionDecision, ProposalCard, ProposalTraceability, ProposalType, Question, QuestionStatus,
    RequiredHumanDecision, Requirement, RequirementStatus, Resolution, ResolutionInput,
    ResolutionMethod, ResolutionStatus, Rule, RuleModality, RuleSeverity, RuleStatus, RuleType,
    ScopeId, Service, ServiceBinding, ServiceBindingType, ServiceEnvironment, ServiceStatus,
    ServiceTier, Source, SourceReference, SourceType, StableId, SuggestedArtifact,
    SuggestedArtifactChange, SynthesisPacket, Thread, ThreadParent, Topic, TopicStatus,
    UncertaintyRating, UnsupportedRecommendation, UnsupportedSpeculation,
};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub struct StateStore {
    pub(crate) layout: ProvenanceLayout,
}

pub struct CreateSourceInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    pub source_type: SourceType,
    pub url: Option<String>,
    pub reference: Option<String>,
    pub commit_pin: Option<String>,
    pub effective_date: Option<i64>,
    pub review_date: Option<i64>,
    pub superseded_by: Option<StableId>,
    pub origin_thread: Option<StableId>,
    pub origin_message: Option<StableId>,
}

pub struct CreateRequirementInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub statement: String,
    pub description: Option<String>,
    pub status: RequirementStatus,
    pub domain_id: Option<StableId>,
    pub origin_thread: Option<StableId>,
    pub origin_message: Option<StableId>,
}

pub struct CreateDomainInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

pub struct AddSourceReferenceInput {
    pub scope_id: ScopeId,
    pub source_id: StableId,
    pub requirement_id: StableId,
    pub clause: Option<String>,
}

pub struct CreateEdgeInput {
    pub scope_id: ScopeId,
    pub edge_type: EdgeType,
    pub from_type: NodeType,
    pub from_id: StableId,
    pub to_type: NodeType,
    pub to_id: StableId,
}

pub struct CreateBoundaryInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub requirement_id: StableId,
    pub statement: String,
    pub source_ref: Option<SourceReference>,
}

pub struct CreateTopicInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub requirement_id: StableId,
    pub title: String,
    pub status: TopicStatus,
    pub links: Vec<ArtifactLink>,
}

pub struct CreateQuestionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub topic_id: StableId,
    pub question: String,
    pub resolution_method: ResolutionMethod,
    pub status: QuestionStatus,
    pub answer: Option<String>,
    pub links: Vec<ArtifactLink>,
    pub resolution_id: Option<StableId>,
}

pub struct UpdateQuestionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub resolution_method: Option<ResolutionMethod>,
    pub status: Option<QuestionStatus>,
    pub links: Option<Vec<ArtifactLink>>,
    pub resolution_id: Option<StableId>,
}

pub struct CreateResolutionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub title: String,
    pub requirement_id: Option<StableId>,
    pub position: String,
    pub rationale: String,
    pub status: ResolutionStatus,
    pub context: Option<String>,
    pub enforcement: Option<String>,
    pub confidence: Option<f64>,
    pub inputs: Vec<ResolutionInput>,
    pub made_by: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<i64>,
    pub superseded_by: Option<StableId>,
    pub origin_thread: Option<StableId>,
    pub origin_message: Option<StableId>,
}

pub struct CreateRuleInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub rule_code: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub requirement_id: Option<StableId>,
    pub resolution_id: Option<StableId>,
    pub statement: String,
    pub status: RuleStatus,
    pub severity: RuleSeverity,
    pub rule_type: Option<RuleType>,
    pub modality: Option<RuleModality>,
    pub confidence: Option<f64>,
    pub extraction_method: Option<String>,
    pub source_document: Option<String>,
    pub source_section: Option<String>,
    pub origin_thread: Option<StableId>,
    pub origin_message: Option<StableId>,
}

pub struct CreateServiceInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub repository: Option<String>,
    pub environment: Option<ServiceEnvironment>,
    pub tier: Option<ServiceTier>,
    pub external_id: Option<String>,
    pub status: ServiceStatus,
}

pub struct CreateServiceBindingInput {
    pub scope_id: ScopeId,
    pub rule_id: StableId,
    pub service_id: StableId,
    pub binding_type: ServiceBindingType,
}

pub struct PostMessageInput {
    pub scope_id: ScopeId,
    pub parent: ThreadParent,
    pub role: MessageRole,
    pub body: String,
}

pub struct CreateContributionInput {
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

pub struct CreateSynthesisPacketInput {
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

pub struct CreateProposalCardInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub proposal_key: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub summary: String,
    pub confidence: Option<f64>,
    pub traceability: ProposalTraceability,
    pub builds_on: Vec<AssertionId>,
    pub duplicate_of: Option<StableId>,
    pub superseded_by: Option<StableId>,
}

pub struct CreateAssertionInput {
    pub scope_id: ScopeId,
    pub id: AssertionId,
    pub proposal_id: StableId,
    pub synthesis_packet_id: StableId,
    pub supporting_claim_ids: Vec<StableId>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdeationLandingBatch {
    pub contributions: Vec<Contribution>,
    pub synthesis_packets: Vec<SynthesisPacket>,
    pub proposals: Vec<ProposalCard>,
    pub assertions: Vec<AssertionRecord>,
    #[serde(default)]
    pub dispositions: Vec<DispositionRecord>,
}

pub struct CreatePromotionDecisionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub proposal_id: StableId,
    pub decision: PromotionDecision,
    pub rationale: String,
    pub actor: PromotionActor,
    pub canonical_artifact: Option<CanonicalArtifact>,
}

#[derive(Debug, serde::Serialize)]
pub struct PostMessageResult {
    pub thread: Thread,
    pub message: Message,
}

impl StateStore {
    pub const fn new(layout: ProvenanceLayout) -> Self {
        Self { layout }
    }
    pub fn manifest(&self) -> anyhow::Result<Manifest> {
        Ok(serde_json::from_str(&std::fs::read_to_string(
            self.layout.manifest_path(),
        )?)?)
    }

    pub fn list_scope_directories(&self) -> anyhow::Result<Vec<String>> {
        let scopes_dir = self.layout.scopes_dir();
        if !scopes_dir.exists() {
            return Ok(Vec::new());
        }

        let mut scope_directories = Vec::new();
        for entry in std::fs::read_dir(scopes_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                scope_directories.push(entry.file_name().into_string().map_err(|name| {
                    anyhow::anyhow!("non-UTF-8 scope directory name: {}", name.to_string_lossy())
                })?);
            }
        }
        scope_directories.sort();
        Ok(scope_directories)
    }

    pub fn list_sources(&self, scope: &ScopeId) -> anyhow::Result<Vec<Source>> {
        read_jsonl(&shards::sources_path(&self.layout, scope))
    }
    pub fn list_requirements(&self, scope: &ScopeId) -> anyhow::Result<Vec<Requirement>> {
        read_jsonl(&shards::requirements_path(&self.layout, scope))
    }
    pub fn list_domains(&self, scope: &ScopeId) -> anyhow::Result<Vec<Domain>> {
        read_jsonl(&shards::domains_path(&self.layout, scope))
    }
    pub fn list_boundaries(&self, scope: &ScopeId) -> anyhow::Result<Vec<Boundary>> {
        read_jsonl(&shards::boundaries_path(&self.layout, scope))
    }
    pub fn list_topics(&self, scope: &ScopeId) -> anyhow::Result<Vec<Topic>> {
        read_jsonl(&shards::topics_path(&self.layout, scope))
    }
    pub fn list_questions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Question>> {
        read_jsonl(&shards::questions_path(&self.layout, scope))
    }
    pub fn list_edges(&self) -> anyhow::Result<Vec<Edge>> {
        read_edge_shards(&self.layout)
    }
    pub fn list_resolutions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Resolution>> {
        read_jsonl(&shards::resolutions_path(&self.layout, scope))
    }
    pub fn list_rules(&self, scope: &ScopeId) -> anyhow::Result<Vec<Rule>> {
        read_jsonl(&shards::rules_path(&self.layout, scope))
    }
    pub fn list_services(&self, scope: &ScopeId) -> anyhow::Result<Vec<Service>> {
        read_jsonl(&shards::services_path(&self.layout, scope))
    }
    pub fn list_service_bindings(&self, scope: &ScopeId) -> anyhow::Result<Vec<ServiceBinding>> {
        read_jsonl(&shards::service_bindings_path(&self.layout, scope))
    }
    pub fn list_threads(&self, scope: &ScopeId) -> anyhow::Result<Vec<Thread>> {
        read_jsonl(&shards::threads_path(&self.layout, scope))
    }
    pub fn list_messages(&self, scope: &ScopeId) -> anyhow::Result<Vec<Message>> {
        read_message_shards(&self.layout, scope)
    }
    pub(crate) fn mutate_jsonl_records<T, R>(
        &self,
        path: &Utf8Path,
        mutate: impl FnOnce(&mut Vec<T>) -> anyhow::Result<R>,
    ) -> anyhow::Result<R>
    where
        T: DeserializeOwned + Serialize,
    {
        let lock_path = self.layout.state_shard_lock_path(path)?;
        crate::jsonl::mutate_jsonl_locked(path, &lock_path, mutate)
    }

    pub(crate) fn with_ideation_lock<R>(
        &self,
        scope: &ScopeId,
        operation: impl FnOnce() -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        crate::jsonl::with_exclusive_lock(&self.layout.ideation_lock_path(scope), operation)
    }
}

pub(crate) fn serde_name<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_value(value)?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("expected string enum serialization"))?
        .to_string())
}

fn read_jsonl<T: DeserializeOwned>(path: &camino::Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

fn read_jsonl_shards<T: DeserializeOwned>(
    shard_paths: Vec<Utf8PathBuf>,
    shard_kind: &str,
) -> anyhow::Result<Vec<T>> {
    let mut records = Vec::new();
    for path in shard_paths {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {shard_kind} shard {}", path.as_str()))?;
        for (index, line) in contents.lines().enumerate() {
            records.push(serde_json::from_str(line).with_context(|| {
                format!(
                    "failed to parse {shard_kind} shard {} line {}",
                    path.as_str(),
                    index + 1
                )
            })?);
        }
    }
    Ok(records)
}

fn read_message_shards(layout: &ProvenanceLayout, scope: &ScopeId) -> anyhow::Result<Vec<Message>> {
    let threads_dir = shards::threads_path(layout, scope)
        .parent()
        .expect("threads path must have a parent")
        .to_path_buf();
    if !threads_dir.exists() {
        return Ok(Vec::new());
    }

    let mut shard_paths = Vec::new();
    for entry in std::fs::read_dir(&threads_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                anyhow::anyhow!("non-UTF-8 message shard path: {}", path.display())
            })?;
            if is_message_month_shard(&path) {
                shard_paths.push(path);
            }
        }
    }
    shard_paths.sort();
    read_jsonl_shards(shard_paths, "message")
}

fn is_message_month_shard(path: &Utf8Path) -> bool {
    let Some(file_name) = path.file_name() else {
        return false;
    };
    let bytes = file_name.as_bytes();
    bytes.len() == "2026-07.jsonl".len()
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && &bytes[7..] == b".jsonl"
}

fn read_edge_shards(layout: &ProvenanceLayout) -> anyhow::Result<Vec<Edge>> {
    let edges_dir = layout.edges_dir();
    if !edges_dir.exists() {
        return Ok(Vec::new());
    }

    let mut shard_paths = Vec::new();
    for entry in std::fs::read_dir(&edges_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|path| anyhow::anyhow!("non-UTF-8 edge shard path: {}", path.display()))?;
            if path.extension() == Some("jsonl") {
                shard_paths.push(path);
            }
        }
    }
    shard_paths.sort();

    read_jsonl_shards(shard_paths, "edge")
}

#[cfg(test)]
mod tests;
