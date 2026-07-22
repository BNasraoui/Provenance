use serde::{Deserialize, Serialize};

use super::ids::{SchemaVersion, ScopeId, StableId};
use super::parsing::normalize_enum_value;
use super::validation::{deserialize_optional_commit_pin, deserialize_optional_confidence};

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
pub enum ResolutionInputType {
    #[serde(rename = "regulatory")]
    Regulatory,
    #[serde(rename = "legal_advice")]
    LegalAdvice,
    #[serde(rename = "commercial")]
    Commercial,
    #[serde(rename = "benchmark")]
    Benchmark,
    #[serde(rename = "technical")]
    Technical,
    #[serde(rename = "incident")]
    Incident,
    #[serde(rename = "source_material")]
    SourceMaterial,
}

impl ResolutionInputType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "regulatory" => Ok(Self::Regulatory),
            "legal_advice" => Ok(Self::LegalAdvice),
            "commercial" => Ok(Self::Commercial),
            "benchmark" => Ok(Self::Benchmark),
            "technical" => Ok(Self::Technical),
            "incident" => Ok(Self::Incident),
            "source_material" => Ok(Self::SourceMaterial),
            _ => anyhow::bail!("resolution input type must be regulatory, legal_advice, commercial, benchmark, technical, incident, or source_material"),
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

const fn empty_array() -> serde_json::Value {
    serde_json::json!([])
}

fn empty_object() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct SourceReference {
    #[serde(alias = "sourceId")]
    pub source_id: StableId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ResolutionInput {
    #[serde(alias = "inputType")]
    pub input_type: ResolutionInputType,
    pub reference: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
