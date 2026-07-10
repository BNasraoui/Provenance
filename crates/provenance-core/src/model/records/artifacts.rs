use serde::{Deserialize, Serialize};

use super::validation::{deserialize_optional_commit_pin, deserialize_optional_confidence};
use crate::model::{
    RequirementStatus, ResolutionInputType, ResolutionStatus, RuleModality, RuleSeverity,
    RuleStatus, RuleType, SchemaVersion, ScopeId, SourceType, StableId,
};

const fn empty_array() -> serde_json::Value {
    serde_json::json!([])
}

fn empty_object() -> serde_json::Value {
    serde_json::json!({})
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
