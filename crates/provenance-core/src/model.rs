mod artifacts;
mod collaboration;
mod graph;
mod ideation;
mod ids;
mod manifest;
mod parsing;
mod services;
mod shaping;
mod validation;

pub use artifacts::{
    Requirement, RequirementStatus, Resolution, ResolutionInput, ResolutionInputType,
    ResolutionStatus, Rule, RuleModality, RuleSeverity, RuleStatus, RuleType, Source,
    SourceReference, SourceType,
};
pub use collaboration::{Message, MessageRole, Thread, ThreadParent, ThreadStatus};
pub use graph::{Edge, EdgeType, NodeType};
pub use ideation::contributions::{
    ClaimChallenge, Contribution, MaterialClaim, SuggestedArtifactChange, UncertaintyRating,
    UnsupportedRecommendation,
};
pub use ideation::dispositions::{
    validate_disposition_intrinsic, CanonicalArtifact, DispositionActor, DispositionRecord,
};
pub use ideation::lifecycle::{
    effective_proposal_state, validate_assertion_intrinsic, validate_ideation_aggregate,
    validate_proposal_intrinsic, Assertion, AssertionId, AssertionRecord, IdeationAggregate,
    LegacyProposalPolicy,
};
pub use ideation::proposals::{Proposal, ProposalCard, ProposalTraceability};
pub use ideation::synthesis::{
    ConsensusFinding, ContestedClaim, EvidenceGap, MinorityObjection, RequiredHumanDecision,
    SuggestedArtifact, SynthesisPacket, UnsupportedSpeculation,
};
pub use ideation::{
    ArtifactChangeType, CanonicalArtifactType, ContributionStance, DispositionDecision,
    EvidenceQuality, IdeationEvidenceReference, IdeationEvidenceType, IdeationTarget,
    IdeationTargetType, IdentityType, PromotionState, ProposalType, SpeculationMarker,
    UncertaintyLevel,
};
pub use ids::{SchemaVersion, ScopeId, StableId};
pub use manifest::{Manifest, RepoPathPrefix, Scope};
pub use services::{
    Domain, Service, ServiceBinding, ServiceBindingType, ServiceEnvironment, ServiceStatus,
    ServiceTier,
};
pub use shaping::{
    ArtifactLink, ArtifactLinkTargetType, Boundary, Question, QuestionStatus, ResolutionMethod,
    Topic, TopicStatus,
};
pub use validation::{
    validate_commit_pin, validate_confidence_score, validate_optional_commit_pin,
    validate_optional_confidence_score,
};

#[cfg(test)]
mod tests;
