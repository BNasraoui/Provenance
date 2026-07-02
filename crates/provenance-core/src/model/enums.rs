use serde::{Deserialize, Serialize};

fn normalize_enum_value(value: &str) -> String {
    value.trim().replace('-', "_").to_ascii_lowercase()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    #[serde(rename = "policy")]
    Policy,
    #[serde(rename = "document")]
    Document,
    #[serde(rename = "legislation")]
    Legislation,
    #[serde(rename = "company_agreement")]
    CompanyAgreement,
    #[serde(rename = "system_state")]
    SystemState,
    #[serde(rename = "external_integration")]
    ExternalIntegration,
    #[serde(rename = "domain_knowledge")]
    DomainKnowledge,
    #[serde(rename = "project_artifact")]
    ProjectArtifact,
    #[serde(rename = "incident")]
    Incident,
    #[serde(rename = "api_spec")]
    ApiSpec,
}

impl SourceType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "policy" => Ok(Self::Policy),
            "document" => Ok(Self::Document),
            "legislation" => Ok(Self::Legislation),
            "company_agreement" => Ok(Self::CompanyAgreement),
            "system_state" => Ok(Self::SystemState),
            "external_integration" => Ok(Self::ExternalIntegration),
            "domain_knowledge" => Ok(Self::DomainKnowledge),
            "project_artifact" => Ok(Self::ProjectArtifact),
            "incident" => Ok(Self::Incident),
            "api_spec" => Ok(Self::ApiSpec),
            _ => anyhow::bail!("source type must be a supported provenance source type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "discovery")]
    Discovery,
    #[serde(rename = "refinement")]
    Refinement,
    #[serde(rename = "resolved")]
    Resolved,
}

impl RequirementStatus {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "active" => Ok(Self::Active),
            "discovery" => Ok(Self::Discovery),
            "refinement" => Ok(Self::Refinement),
            "resolved" => Ok(Self::Resolved),
            _ => anyhow::bail!(
                "requirement status must be active, discovery, refinement, or resolved"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "requirement")]
    Requirement,
    #[serde(rename = "resolution")]
    Resolution,
    #[serde(rename = "rule")]
    Rule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdeationTargetType {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "requirement")]
    Requirement,
    #[serde(rename = "resolution")]
    Resolution,
    #[serde(rename = "rule")]
    Rule,
    #[serde(rename = "topic")]
    Topic,
    #[serde(rename = "question")]
    Question,
    #[serde(rename = "domain")]
    Domain,
}

impl IdeationTargetType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "source" => Ok(Self::Source),
            "requirement" => Ok(Self::Requirement),
            "resolution" => Ok(Self::Resolution),
            "rule" => Ok(Self::Rule),
            "topic" => Ok(Self::Topic),
            "question" => Ok(Self::Question),
            "domain" => Ok(Self::Domain),
            _ => anyhow::bail!(
                "target type must be source, requirement, resolution, rule, topic, question, or domain"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalArtifactType {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "requirement")]
    Requirement,
    #[serde(rename = "resolution")]
    Resolution,
    #[serde(rename = "rule")]
    Rule,
}

impl CanonicalArtifactType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "source" => Ok(Self::Source),
            "requirement" => Ok(Self::Requirement),
            "resolution" => Ok(Self::Resolution),
            "rule" => Ok(Self::Rule),
            _ => anyhow::bail!(
                "canonical artifact type must be source, requirement, resolution, or rule"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityType {
    #[serde(rename = "human")]
    Human,
    #[serde(rename = "agent")]
    Agent,
    #[serde(rename = "service")]
    Service,
}

impl IdentityType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "human" => Ok(Self::Human),
            "agent" => Ok(Self::Agent),
            "service" => Ok(Self::Service),
            _ => anyhow::bail!("identity type must be human, agent, or service"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdeationEvidenceType {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "artifact")]
    Artifact,
    #[serde(rename = "thread_message")]
    ThreadMessage,
    #[serde(rename = "domain_knowledge")]
    DomainKnowledge,
    #[serde(rename = "unsupported")]
    Unsupported,
    #[serde(rename = "exploratory")]
    Exploratory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContributionStance {
    #[serde(rename = "support")]
    Support,
    #[serde(rename = "oppose")]
    Oppose,
    #[serde(rename = "mixed")]
    Mixed,
    #[serde(rename = "needs_more_evidence")]
    NeedsMoreEvidence,
}

impl ContributionStance {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "support" => Ok(Self::Support),
            "oppose" => Ok(Self::Oppose),
            "mixed" => Ok(Self::Mixed),
            "needs_more_evidence" => Ok(Self::NeedsMoreEvidence),
            _ => anyhow::bail!("stance must be support, oppose, mixed, or needs_more_evidence"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactChangeType {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "remove")]
    Remove,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeculationMarker {
    #[serde(rename = "unsupported")]
    Unsupported,
    #[serde(rename = "exploratory")]
    Exploratory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UncertaintyLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

impl UncertaintyLevel {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => anyhow::bail!("uncertainty level must be low, medium, or high"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceQuality {
    #[serde(rename = "strong")]
    Strong,
    #[serde(rename = "mixed")]
    Mixed,
    #[serde(rename = "weak")]
    Weak,
    #[serde(rename = "unsupported")]
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    #[serde(rename = "requirement_candidate")]
    RequirementCandidate,
    #[serde(rename = "resolution_candidate")]
    ResolutionCandidate,
    #[serde(rename = "rule_candidate")]
    RuleCandidate,
    #[serde(rename = "source_gap")]
    SourceGap,
    #[serde(rename = "question")]
    Question,
    #[serde(rename = "no_action")]
    NoAction,
}

impl ProposalType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "requirement_candidate" => Ok(Self::RequirementCandidate),
            "resolution_candidate" => Ok(Self::ResolutionCandidate),
            "rule_candidate" => Ok(Self::RuleCandidate),
            "source_gap" => Ok(Self::SourceGap),
            "question" => Ok(Self::Question),
            "no_action" => Ok(Self::NoAction),
            _ => anyhow::bail!("proposal type must be requirement_candidate, resolution_candidate, rule_candidate, source_gap, question, or no_action"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionState {
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "deferred")]
    Deferred,
    #[serde(rename = "duplicate")]
    Duplicate,
    #[serde(rename = "superseded")]
    Superseded,
}

impl PromotionState {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "deferred" => Ok(Self::Deferred),
            "duplicate" => Ok(Self::Duplicate),
            "superseded" => Ok(Self::Superseded),
            _ => anyhow::bail!(
                "promotion state must be proposed, accepted, rejected, deferred, duplicate, or superseded"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionDecision {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "deferred")]
    Deferred,
}

impl PromotionDecision {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "deferred" => Ok(Self::Deferred),
            _ => anyhow::bail!("promotion decision must be accepted, rejected, or deferred"),
        }
    }
}

impl NodeType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match value {
            "source" => Ok(Self::Source),
            "requirement" => Ok(Self::Requirement),
            "resolution" => Ok(Self::Resolution),
            "rule" => Ok(Self::Rule),
            _ => anyhow::bail!("parent type must be source, requirement, resolution, or rule"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    #[serde(rename = "references")]
    References,
    #[serde(rename = "refines_into")]
    RefinesInto,
    #[serde(rename = "depends_on")]
    DependsOn,
    #[serde(rename = "contradicts")]
    Contradicts,
    #[serde(rename = "supersedes")]
    Supersedes,
    #[serde(rename = "needs")]
    Needs,
    #[serde(rename = "resolves")]
    Resolves,
    #[serde(rename = "spawns")]
    Spawns,
    #[serde(rename = "produces")]
    Produces,
}

impl EdgeType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match value {
            "references" => Ok(Self::References),
            "refines_into" => Ok(Self::RefinesInto),
            "depends_on" => Ok(Self::DependsOn),
            "contradicts" => Ok(Self::Contradicts),
            "supersedes" => Ok(Self::Supersedes),
            "needs" => Ok(Self::Needs),
            "resolves" => Ok(Self::Resolves),
            "spawns" => Ok(Self::Spawns),
            "produces" => Ok(Self::Produces),
            _ => anyhow::bail!("edge type must be a supported provenance edge type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "revised")]
    Revised,
    #[serde(rename = "superseded")]
    Superseded,
    #[serde(rename = "abandoned")]
    Abandoned,
}

impl ResolutionStatus {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "draft" => Ok(Self::Draft),
            "review" => Ok(Self::Review),
            "proposed" => Ok(Self::Proposed),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "revised" => Ok(Self::Revised),
            "superseded" => Ok(Self::Superseded),
            "abandoned" => Ok(Self::Abandoned),
            _ => anyhow::bail!("resolution status must be draft, review, proposed, approved, rejected, revised, superseded, or abandoned"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "deprecated")]
    Deprecated,
    #[serde(rename = "archived")]
    Archived,
}

impl RuleStatus {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "draft" => Ok(Self::Draft),
            "review" => Ok(Self::Review),
            "active" => Ok(Self::Active),
            "deprecated" => Ok(Self::Deprecated),
            "archived" => Ok(Self::Archived),
            _ => {
                anyhow::bail!("rule status must be draft, review, active, deprecated, or archived")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSeverity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

impl RuleSeverity {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => anyhow::bail!("severity must be low, medium, high, or critical"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    #[serde(rename = "business")]
    Business,
    #[serde(rename = "functional")]
    Functional,
    #[serde(rename = "technical")]
    Technical,
}

impl RuleType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "business" => Ok(Self::Business),
            "functional" => Ok(Self::Functional),
            "technical" => Ok(Self::Technical),
            _ => anyhow::bail!("rule type must be business, functional, or technical"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleModality {
    #[serde(rename = "obligation")]
    Obligation,
    #[serde(rename = "prohibition")]
    Prohibition,
    #[serde(rename = "necessity")]
    Necessity,
}

impl RuleModality {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "obligation" => Ok(Self::Obligation),
            "prohibition" => Ok(Self::Prohibition),
            "necessity" => Ok(Self::Necessity),
            _ => anyhow::bail!("rule modality must be obligation, prohibition, or necessity"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "archived")]
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

impl MessageRole {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match value {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            _ => anyhow::bail!("role must be user, assistant, or system"),
        }
    }
}
