use serde::{Deserialize, Serialize};

use crate::model::{EdgeType, NodeType, SchemaVersion, ScopeId, StableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub edge_type: EdgeType,
    pub from_type: NodeType,
    pub from_id: StableId,
    pub to_type: NodeType,
    pub to_id: StableId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl Edge {
    pub fn stable_id(
        edge_type: EdgeType,
        from_type: NodeType,
        from_id: &StableId,
        to_type: NodeType,
        to_id: &StableId,
    ) -> anyhow::Result<StableId> {
        StableId::new(
            format!(
                "{:?}_{:?}_{}_to_{:?}_{}",
                edge_type,
                from_type,
                from_id.as_str(),
                to_type,
                to_id.as_str()
            )
            .to_ascii_lowercase(),
        )
    }
}
