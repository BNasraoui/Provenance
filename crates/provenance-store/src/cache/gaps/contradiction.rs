use super::{graph_query::GraphQuery, GapItem, GapKind};
use provenance_core::{EdgeType, NodeType, StableId};
use std::collections::BTreeSet;

pub(super) fn add_gaps(query: &GraphQuery<'_, '_>, gaps: &mut Vec<GapItem>) {
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    for edge in query.edges().filter(|edge| {
        edge.edge_type == EdgeType::Contradicts
            && edge.from_type == NodeType::Requirement
            && edge.to_type == NodeType::Requirement
            && query.requirement_exists(&edge.from_id)
            && query.requirement_exists(&edge.to_id)
    }) {
        let pair = ordered_pair(&edge.from_id, &edge.to_id);
        if !seen.insert(pair) || is_resolved(query, &edge.from_id, &edge.to_id) {
            continue;
        }
        gaps.push(
            GapItem::new(
                GapKind::UnresolvedContradictsPair,
                NodeType::Requirement,
                &edge.from_id,
                "unresolved `contradicts` pair",
            )
            .with_related(NodeType::Requirement, &edge.to_id),
        );
    }
}

fn is_resolved(query: &GraphQuery<'_, '_>, left_id: &StableId, right_id: &StableId) -> bool {
    if query.edge_exists(
        EdgeType::Supersedes,
        NodeType::Requirement,
        left_id,
        NodeType::Requirement,
        right_id,
    ) || query.edge_exists(
        EdgeType::Supersedes,
        NodeType::Requirement,
        right_id,
        NodeType::Requirement,
        left_id,
    ) {
        return true;
    }
    let left_resolutions: BTreeSet<&str> = query
        .resolving_resolutions(left_id)
        .into_iter()
        .map(|resolution| resolution.id.as_str())
        .collect();
    query
        .resolving_resolutions(right_id)
        .into_iter()
        .any(|resolution| left_resolutions.contains(resolution.id.as_str()))
}

fn ordered_pair<'a>(left: &'a StableId, right: &'a StableId) -> (&'a str, &'a str) {
    if left.as_str() <= right.as_str() {
        (left.as_str(), right.as_str())
    } else {
        (right.as_str(), left.as_str())
    }
}
