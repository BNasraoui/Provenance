pub mod coverage;
pub mod edge_validation;
pub mod model;
pub mod scope;
pub mod threads;

pub use model::{
    ensure_authoritative_actor, validate_commit_pin, validate_confidence_score,
    validate_ideation_aggregate, validate_optional_commit_pin, validate_optional_confidence_score,
    validate_proposal_intrinsic, ArtifactChangeType, ArtifactLink, ArtifactLinkTargetType,
    AssertionId, AssertionRecord, Boundary, CanonicalArtifact, CanonicalArtifactType,
    ClaimChallenge, ConsensusFinding, ContestedClaim, Contribution, ContributionStance,
    DispositionRecord, Domain, Edge, EdgeType, EvidenceGap, EvidenceQuality, IdeationAggregate,
    IdeationEvidenceReference, IdeationEvidenceType, IdeationTarget, IdeationTargetType,
    IdentityType, Manifest, MaterialClaim, Message, MessageRole, MinorityObjection, NodeType,
    PromotionActor, PromotionDecision, PromotionState, ProposalCard, ProposalTraceability,
    ProposalType, ProposalView, Question, QuestionStatus, RepoPathPrefix, RequiredHumanDecision,
    Requirement, RequirementStatus, Resolution, ResolutionInput, ResolutionInputType,
    ResolutionMethod, ResolutionStatus, Rule, RuleModality, RuleSeverity, RuleStatus, RuleType,
    SchemaVersion, Scope, ScopeId, Service, ServiceBinding, ServiceBindingType, ServiceEnvironment,
    ServiceStatus, ServiceTier, Source, SourceReference, SourceType, SpeculationMarker, StableId,
    SuggestedArtifact, SuggestedArtifactChange, SynthesisPacket, Thread, ThreadParent,
    ThreadStatus, Topic, TopicStatus, UncertaintyLevel, UncertaintyRating,
    UnsupportedRecommendation, UnsupportedSpeculation,
};
