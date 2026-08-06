use provenance_core::{
    ArtifactLink, CanonicalArtifact, ClaimChallenge, ConsensusFinding, ContestedClaim,
    ContributionStance, DispositionActor, DispositionDecision, EdgeType, EvidenceGap,
    IdeationEvidenceReference, IdeationTarget, MaterialClaim, MessageRole, MinorityObjection,
    NodeType, PromotionState, ProposalTraceability, ProposalType, QuestionStatus,
    RequiredHumanDecision, RequirementStatus, ResolutionInput, ResolutionMethod, ResolutionStatus,
    RuleModality, RuleSeverity, RuleStatus, RuleType, ScopeId, ServiceBindingType,
    ServiceEnvironment, ServiceStatus, ServiceTier, SourceReference, SourceType, StableId,
    SuggestedArtifact, SuggestedArtifactChange, ThreadParent, TopicStatus, UncertaintyRating,
    UnsupportedRecommendation, UnsupportedSpeculation,
};

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
