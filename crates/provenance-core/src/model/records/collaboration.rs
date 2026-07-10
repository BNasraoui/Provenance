use serde::{Deserialize, Serialize};

use crate::model::{MessageRole, NodeType, SchemaVersion, ScopeId, StableId, ThreadStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadParent {
    pub node_type: NodeType,
    pub node_id: StableId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thread {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub parent: ThreadParent,
    pub status: ThreadStatus,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub thread_id: StableId,
    pub role: MessageRole,
    #[serde(alias = "content")]
    pub body: String,
    pub created_at: i64,
    #[serde(default, alias = "aiMetadata", skip_serializing_if = "Option::is_none")]
    pub ai_metadata: Option<serde_json::Value>,
}
