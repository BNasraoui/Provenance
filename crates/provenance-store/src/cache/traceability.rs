use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{Edge, EdgeType, NodeType, Requirement, Resolution, Rule, Source};

#[derive(Debug, serde::Serialize)]
pub struct TraceabilityView {
    pub rule: Rule,
    pub resolutions: Vec<Resolution>,
    pub requirements: Vec<Requirement>,
    pub sources: Vec<Source>,
    pub edges: Vec<Edge>,
}

pub fn trace_rule(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    rule_id: &provenance_core::StableId,
) -> anyhow::Result<TraceabilityView> {
    let snapshot = StateStore::new(layout.clone()).scope_snapshot(scope)?;
    let rule = snapshot
        .rules
        .into_iter()
        .find(|rule| rule.id == *rule_id)
        .ok_or_else(|| anyhow::anyhow!("rule not found"))?;
    let edges: Vec<Edge> = snapshot.edges;
    let resolution_ids: Vec<_> = edges
        .iter()
        .filter(|edge| {
            edge.edge_type == EdgeType::Produces
                && edge.from_type == NodeType::Resolution
                && edge.to_type == NodeType::Rule
                && edge.to_id == *rule_id
        })
        .map(|edge| edge.from_id.clone())
        .collect();
    let requirement_ids: Vec<_> = edges
        .iter()
        .filter(|edge| {
            (edge.edge_type == EdgeType::Produces
                && edge.from_type == NodeType::Requirement
                && edge.to_type == NodeType::Rule
                && edge.to_id == *rule_id)
                || (edge.edge_type == EdgeType::Resolves
                    && edge.from_type == NodeType::Resolution
                    && resolution_ids.iter().any(|id| id == &edge.from_id))
        })
        .map(|edge| edge.to_id.clone())
        .collect();
    let source_ids: Vec<_> = edges
        .iter()
        .filter(|edge| {
            edge.edge_type == EdgeType::References
                && edge.from_type == NodeType::Source
                && requirement_ids.iter().any(|id| id == &edge.to_id)
        })
        .map(|edge| edge.from_id.clone())
        .collect();
    let resolutions = snapshot
        .resolutions
        .into_iter()
        .filter(|resolution| resolution_ids.iter().any(|id| id == &resolution.id))
        .collect();
    let requirements = snapshot
        .requirements
        .into_iter()
        .filter(|requirement| requirement_ids.iter().any(|id| id == &requirement.id))
        .collect();
    let sources = snapshot
        .sources
        .into_iter()
        .filter(|source| source_ids.iter().any(|id| id == &source.id))
        .collect();
    Ok(TraceabilityView {
        rule,
        resolutions,
        requirements,
        sources,
        edges,
    })
}
