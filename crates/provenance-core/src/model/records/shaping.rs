use serde::{Deserialize, Serialize};

use crate::model::{
    ArtifactLinkTargetType, QuestionStatus, ResolutionMethod, SchemaVersion, ScopeId,
    SourceReference, StableId, TopicStatus,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLink {
    #[serde(alias = "targetType")]
    pub target_type: ArtifactLinkTargetType,
    #[serde(alias = "targetId")]
    pub target_id: StableId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boundary {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "requirementId")]
    pub requirement_id: StableId,
    pub statement: String,
    #[serde(default, alias = "sourceRef", skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<SourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "requirementId")]
    pub requirement_id: StableId,
    pub title: String,
    pub status: TopicStatus,
    #[serde(default, alias = "claimedBy", skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    #[serde(default, alias = "claimedAt", skip_serializing_if = "Option::is_none")]
    pub claimed_at: Option<i64>,
    #[serde(default)]
    pub links: Vec<ArtifactLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "topicId")]
    pub topic_id: StableId,
    #[serde(alias = "requirementId")]
    pub requirement_id: StableId,
    pub question: String,
    /// The verb that resolves this question, chosen when the question is minted.
    #[serde(alias = "resolutionMethod")]
    pub resolution_method: ResolutionMethod,
    pub status: QuestionStatus,
    #[serde(default, alias = "claimedBy", skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    #[serde(default, alias = "claimedAt", skip_serializing_if = "Option::is_none")]
    pub claimed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(default)]
    pub links: Vec<ArtifactLink>,
    #[serde(
        default,
        alias = "resolutionId",
        skip_serializing_if = "Option::is_none"
    )]
    pub resolution_id: Option<StableId>,
}
