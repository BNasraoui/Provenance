use super::references::node_type_name;
use provenance_core::{NodeType, ScopeId, StableId};
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct CheckIndex {
    global_nodes: BTreeSet<(String, String)>,
    scoped_nodes: BTreeSet<(String, String, String)>,
}

impl CheckIndex {
    pub(super) fn add_node(&mut self, scope_id: &ScopeId, node_type: &str, id: &StableId) {
        let node_type = node_type.to_string();
        let id = id.as_str().to_string();
        self.global_nodes.insert((node_type.clone(), id.clone()));
        self.scoped_nodes
            .insert((scope_id.as_str().to_string(), node_type, id));
    }

    pub(super) fn has_global_node(&self, node_type: NodeType, id: &StableId) -> bool {
        self.global_nodes.contains(&(
            node_type_name(node_type).to_string(),
            id.as_str().to_string(),
        ))
    }

    pub(super) fn has_scoped_node(
        &self,
        scope_id: &ScopeId,
        node_type: &str,
        id: &StableId,
    ) -> bool {
        self.scoped_nodes.contains(&(
            scope_id.as_str().to_string(),
            node_type.to_string(),
            id.as_str().to_string(),
        ))
    }
}
