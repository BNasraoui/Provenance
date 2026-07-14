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
pub use ideation::promotions::{CanonicalArtifact, PromotionActor, PromotionDecisionRecord};
pub use ideation::proposals::{ProposalCard, ProposalTraceability};
pub use ideation::synthesis::{
    ConsensusFinding, ContestedClaim, EvidenceGap, MinorityObjection, RequiredHumanDecision,
    SuggestedArtifact, SynthesisPacket, UnsupportedSpeculation,
};
pub use ideation::{
    ArtifactChangeType, CanonicalArtifactType, ContributionStance, EvidenceQuality,
    IdeationEvidenceReference, IdeationEvidenceType, IdeationTarget, IdeationTargetType,
    IdentityType, PromotionDecision, PromotionState, ProposalType, SpeculationMarker,
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
    validate_commit_pin, validate_confidence_score, validate_evidence_references,
    validate_optional_commit_pin, validate_optional_confidence_score, validate_record_scope,
    validate_unique_ids,
};

#[cfg(test)]
mod tests;
