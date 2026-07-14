use super::index::CheckIndex;
use super::references::node_type_name;
use provenance_core::edge_validation::validate_edge_endpoint;
use provenance_store::state_store::StateStore;
use std::collections::BTreeSet;

pub(super) fn validate(
    store: &StateStore,
    manifest_scopes: &BTreeSet<String>,
    index: &CheckIndex,
    dangling: &mut Vec<String>,
) -> anyhow::Result<()> {
    for edge in store.list_edges()? {
        if !manifest_scopes.contains(edge.scope_id.as_str()) {
            dangling.push(format!(
                "edge {} is in unknown scope {}",
                edge.id.as_str(),
                edge.scope_id.as_str()
            ));
        }
        if let Err(error) = validate_edge_endpoint(edge.edge_type, edge.from_type, edge.to_type) {
            dangling.push(format!(
                "edge {} has invalid endpoint: {error}",
                edge.id.as_str()
            ));
        }
        if !index.has_global_node(edge.from_type, &edge.from_id) {
            dangling.push(format!(
                "edge {} has dangling reference: from {} {}",
                edge.id.as_str(),
                node_type_name(edge.from_type),
                edge.from_id.as_str()
            ));
        }
        if !index.has_global_node(edge.to_type, &edge.to_id) {
            dangling.push(format!(
                "edge {} has dangling reference: to {} {}",
                edge.id.as_str(),
                node_type_name(edge.to_type),
                edge.to_id.as_str()
            ));
        }
    }
    Ok(())
}
