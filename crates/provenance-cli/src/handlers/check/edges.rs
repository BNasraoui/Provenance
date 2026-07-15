use super::index::CheckIndex;
use super::references::node_type_name;
use provenance_core::edge_validation::validate_edge_endpoint;
use std::collections::BTreeSet;

pub(super) fn validate(
    edges: &[provenance_core::Edge],
    manifest_scopes: &BTreeSet<String>,
    index: &CheckIndex,
    dangling: &mut Vec<String>,
) {
    for edge in edges {
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
}
