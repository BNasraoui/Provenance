mod domain_service_writers;
mod ideation_batches;
mod ideation_writers;
mod proposal_surfaces;
mod readers;
mod rule_writers;
mod shaping_writers;
mod thread_writers;
mod writers;

pub use proposal_surfaces::{ProposalDemand, ProposalSurfaceReason, SurfacedProposal};

use crate::{layout::ProvenanceLayout, shards};
use ideation_batches::overlay_records;
use provenance_core::{
    ArtifactLink, AssertionRecord, Boundary, CanonicalArtifact, ClaimChallenge, ConsensusFinding,
    ContestedClaim, Contribution, ContributionStance, DispositionActor, DispositionDecision,
    DispositionRecord, Domain, Edge, EdgeType, EvidenceGap, IdeationEvidenceReference,
    IdeationTarget, Manifest, MaterialClaim, Message, MessageRole, MinorityObjection, NodeType,
    PromotionState, ProposalCard, ProposalTraceability, ProposalType, Question, QuestionStatus,
    RequiredHumanDecision, Requirement, RequirementStatus, Resolution, ResolutionInput,
    ResolutionMethod, ResolutionStatus, Rule, RuleModality, RuleSeverity, RuleStatus, RuleType,
    SchemaVersion, Scope, ScopeId, Service, ServiceBinding, ServiceBindingType, ServiceEnvironment,
    ServiceStatus, ServiceTier, Source, SourceReference, SourceType, StableId, SuggestedArtifact,
    SuggestedArtifactChange, SynthesisPacket, Thread, ThreadParent, Topic, TopicStatus,
    UncertaintyRating, UnsupportedRecommendation, UnsupportedSpeculation,
};
use readers::{
    deserialize_closed, read_edge_shards, read_jsonl, read_jsonl_closed, read_legacy_dispositions,
    read_message_shards,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestProjection {
    schema_version: SchemaVersion,
    scopes: Vec<serde_json::Value>,
    #[serde(default, rename = "disposition_actor_ids")]
    _disposition_actor_ids: Vec<String>,
}

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
    pub builds_on: Vec<provenance_core::AssertionId>,
    pub promotion_state: PromotionState,
    pub duplicate_of: Option<StableId>,
    pub superseded_by: Option<StableId>,
}

pub struct CreateDispositionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub proposal_id: StableId,
    pub decision: DispositionDecision,
    pub rationale: String,
    pub actor: DispositionActor,
    pub canonical_artifact: Option<CanonicalArtifact>,
}

pub struct CreateAssertionInput {
    pub scope_id: ScopeId,
    pub id: provenance_core::AssertionId,
    pub proposal_id: StableId,
    pub synthesis_packet_id: StableId,
    pub supporting_claim_ids: Vec<StableId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeationLandingBatch {
    #[serde(default)]
    pub contributions: Vec<Contribution>,
    #[serde(default)]
    pub synthesis_packets: Vec<SynthesisPacket>,
    #[serde(default)]
    pub proposals: Vec<ProposalCard>,
    #[serde(default)]
    pub assertions: Vec<AssertionRecord>,
    #[serde(default)]
    pub dispositions: Vec<DispositionRecord>,
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
        self.with_repository_publication(|| {
            Ok(serde_json::from_str(&std::fs::read_to_string(
                self.layout.manifest_path(),
            )?)?)
        })
    }

    pub(crate) fn closed_manifest_scope(
        &self,
        scope: &ScopeId,
    ) -> anyhow::Result<(SchemaVersion, Option<Scope>)> {
        self.with_repository_publication(|| {
            let manifest: ManifestProjection =
                deserialize_closed(&std::fs::read_to_string(self.layout.manifest_path())?)?;
            let selected = manifest
                .scopes
                .into_iter()
                .find(|candidate| {
                    candidate.get("id").and_then(serde_json::Value::as_str) == Some(scope.as_str())
                })
                .map(|candidate| deserialize_closed(&serde_json::to_string(&candidate)?))
                .transpose()?;
            Ok((manifest.schema_version, selected))
        })
    }

    pub fn list_scope_directories(&self) -> anyhow::Result<Vec<String>> {
        self.with_repository_publication(|| {
            let scopes_dir = self.layout.scopes_dir();
            if !scopes_dir.exists() {
                return Ok(Vec::new());
            }

            let mut scope_directories = Vec::new();
            for entry in std::fs::read_dir(scopes_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    scope_directories.push(entry.file_name().into_string().map_err(|name| {
                        anyhow::anyhow!(
                            "non-UTF-8 scope directory name: {}",
                            name.to_string_lossy()
                        )
                    })?);
                }
            }
            scope_directories.sort();
            Ok(scope_directories)
        })
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
        read_edge_shards(&self.layout, None)
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
    pub(crate) fn closed_sources(&self, scope: &ScopeId) -> anyhow::Result<Vec<Source>> {
        read_jsonl_closed(&shards::sources_path(&self.layout, scope))
    }
    pub(crate) fn closed_requirements(&self, scope: &ScopeId) -> anyhow::Result<Vec<Requirement>> {
        read_jsonl_closed(&shards::requirements_path(&self.layout, scope))
    }
    pub(crate) fn closed_domains(&self, scope: &ScopeId) -> anyhow::Result<Vec<Domain>> {
        read_jsonl_closed(&shards::domains_path(&self.layout, scope))
    }
    pub(crate) fn closed_boundaries(&self, scope: &ScopeId) -> anyhow::Result<Vec<Boundary>> {
        read_jsonl_closed(&shards::boundaries_path(&self.layout, scope))
    }
    pub(crate) fn closed_topics(&self, scope: &ScopeId) -> anyhow::Result<Vec<Topic>> {
        read_jsonl_closed(&shards::topics_path(&self.layout, scope))
    }
    pub(crate) fn closed_questions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Question>> {
        read_jsonl_closed(&shards::questions_path(&self.layout, scope))
    }
    pub(crate) fn closed_resolutions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Resolution>> {
        read_jsonl_closed(&shards::resolutions_path(&self.layout, scope))
    }
    pub(crate) fn closed_rules(&self, scope: &ScopeId) -> anyhow::Result<Vec<Rule>> {
        read_jsonl_closed(&shards::rules_path(&self.layout, scope))
    }
    pub(crate) fn closed_services(&self, scope: &ScopeId) -> anyhow::Result<Vec<Service>> {
        read_jsonl_closed(&shards::services_path(&self.layout, scope))
    }
    pub(crate) fn closed_service_bindings(
        &self,
        scope: &ScopeId,
    ) -> anyhow::Result<Vec<ServiceBinding>> {
        read_jsonl_closed(&shards::service_bindings_path(&self.layout, scope))
    }
    pub(crate) fn closed_edges(&self, scope: &ScopeId) -> anyhow::Result<Vec<Edge>> {
        read_edge_shards(&self.layout, Some(scope))
    }
    pub fn list_threads(&self, scope: &ScopeId) -> anyhow::Result<Vec<Thread>> {
        read_jsonl(&shards::threads_path(&self.layout, scope))
    }
    pub fn list_messages(&self, scope: &ScopeId) -> anyhow::Result<Vec<Message>> {
        read_message_shards(&self.layout, scope)
    }
    pub fn list_contributions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Contribution>> {
        let mut records = read_jsonl(&shards::contributions_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay_records(&mut records, batch.contributions, |record| {
                record.id.as_str()
            });
        }
        Ok(records)
    }
    pub fn list_synthesis_packets(&self, scope: &ScopeId) -> anyhow::Result<Vec<SynthesisPacket>> {
        let mut records = read_jsonl(&shards::synthesis_packets_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay_records(&mut records, batch.synthesis_packets, |record| {
                record.id.as_str()
            });
        }
        Ok(records)
    }
    pub fn list_proposal_cards(&self, scope: &ScopeId) -> anyhow::Result<Vec<ProposalCard>> {
        self.project_proposal_cards(scope, || Ok(()))
    }
    fn project_proposal_cards(
        &self,
        scope: &ScopeId,
        after_validation: impl FnOnce() -> anyhow::Result<()>,
    ) -> anyhow::Result<Vec<ProposalCard>> {
        self.with_repository_publication(|| {
            self.validate_ideation_scope(scope)?;
            after_validation()?;
            let assertions = self.list_assertion_records(scope)?;
            let dispositions = self.list_dispositions(scope)?;
            Ok(self
                .list_proposal_definitions(scope)?
                .into_iter()
                .map(|mut proposal| {
                    proposal.promotion_state = provenance_core::effective_proposal_state(
                        &proposal,
                        &assertions,
                        &dispositions,
                    );
                    proposal
                })
                .collect())
        })
    }
    pub fn list_proposal_definitions(&self, scope: &ScopeId) -> anyhow::Result<Vec<ProposalCard>> {
        let mut records = read_jsonl(&shards::proposal_cards_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay_records(&mut records, batch.proposals, |record| record.id.as_str());
        }
        Ok(records)
    }
    pub fn list_dispositions(&self, scope: &ScopeId) -> anyhow::Result<Vec<DispositionRecord>> {
        let mut records = read_jsonl(&shards::dispositions_path(&self.layout, scope))?;
        records.extend(read_legacy_dispositions(
            &shards::legacy_promotion_decisions_path(&self.layout, scope),
        )?);
        for batch in self.list_ideation_landings(scope)? {
            overlay_records(&mut records, batch.dispositions, |record| {
                record.id.as_str()
            });
        }
        Ok(records)
    }
    pub fn list_assertion_records(&self, scope: &ScopeId) -> anyhow::Result<Vec<AssertionRecord>> {
        let mut records = read_jsonl(&shards::assertion_records_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay_records(&mut records, batch.assertions, |record| record.id.as_str());
        }
        Ok(records)
    }
}

pub(crate) fn serde_name<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_value(value)?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("expected string enum serialization"))?
        .to_string())
}

#[cfg(test)]
mod tests;
