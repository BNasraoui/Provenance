pub mod coverage;
pub mod edge_validation;
pub mod model;
pub mod scope;
pub mod threads;

pub use model::{
    disposition_requires_prior_assertion, effective_proposal_state,
    ensure_supported_schema_version, is_shipped_legacy_disposition_audit,
    packet_qualifies_proposal, validate_assertion_intrinsic, validate_commit_pin,
    validate_confidence_score, validate_disposition_admissible, validate_disposition_intrinsic,
    validate_ideation_aggregate, validate_optional_commit_pin, validate_optional_confidence_score,
    validate_proposal_intrinsic, validate_resolution_input_content, ArtifactChangeType,
    ArtifactLink, ArtifactLinkTargetType, Assertion, AssertionId, AssertionRecord, Boundary,
    CanonicalArtifact, CanonicalArtifactType, ClaimChallenge, ConsensusFinding, ContestedClaim,
    Contribution, ContributionStance, DispositionActor, DispositionDecision, DispositionRecord,
    Domain, Edge, EdgeType, EvidenceGap, EvidenceQuality, ExternalActionCorrelation,
    IdeationAggregate, IdeationEvidenceReference, IdeationEvidenceType, IdeationTarget,
    IdeationTargetType, IdentityType, LegacyProposalPolicy, Manifest, MaterialClaim, Message,
    MessageRole, MinorityObjection, NodeType, PromotionState, Proposal, ProposalCard,
    ProposalTraceability, ProposalType, Question, QuestionStatus, RepoPathPrefix,
    RequiredHumanDecision, Requirement, RequirementStatus, Resolution, ResolutionInput,
    ResolutionInputType, ResolutionMethod, ResolutionStatus, Rule, RuleSeverity, RuleStatus,
    SchemaVersion, Scope, ScopeId, Source, SourceReference, SourceType, SpeculationMarker,
    StableId, SuggestedArtifact, SuggestedArtifactChange, SynthesisPacket, Thread, ThreadParent,
    ThreadStatus, Topic, TopicStatus, UncertaintyLevel, UncertaintyRating,
    UnsupportedRecommendation, UnsupportedSpeculation, SUPPORTED_SCHEMA_VERSION,
};
