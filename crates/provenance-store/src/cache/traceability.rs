use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{
    Edge, EdgeType, NodeType, Requirement, RequirementStatus, Resolution, Rule, Source,
};

#[derive(Debug, serde::Serialize)]
pub struct TraceabilityView {
    pub rule: Rule,
    pub resolutions: Vec<Resolution>,
    pub requirements: Vec<Requirement>,
    pub sources: Vec<Source>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, serde::Serialize)]
pub struct GapItem {
    pub requirement_id: String,
    pub reason: String,
}

pub fn trace_rule(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    rule_id: &provenance_core::StableId,
) -> anyhow::Result<TraceabilityView> {
    let store = StateStore::new(layout.clone());
    let rule = store
        .list_rules(scope)?
        .into_iter()
        .find(|rule| rule.id == *rule_id)
        .ok_or_else(|| anyhow::anyhow!("rule not found"))?;
    let edges: Vec<Edge> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| edge.scope_id == *scope)
        .collect();
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
    let resolutions = store
        .list_resolutions(scope)?
        .into_iter()
        .filter(|resolution| resolution_ids.iter().any(|id| id == &resolution.id))
        .collect();
    let requirements = store
        .list_requirements(scope)?
        .into_iter()
        .filter(|requirement| requirement_ids.iter().any(|id| id == &requirement.id))
        .collect();
    let sources = store
        .list_sources(scope)?
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

pub fn find_gaps(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<Vec<GapItem>> {
    let store = StateStore::new(layout.clone());
    let edges = store.list_edges()?;
    Ok(store
        .list_requirements(scope)?
        .into_iter()
        .filter(|requirement| {
            (requirement.status == RequirementStatus::Resolved
                || edges.iter().any(|edge| {
                    edge.scope_id == *scope
                        && edge.edge_type == EdgeType::Resolves
                        && edge.to_type == NodeType::Requirement
                        && edge.to_id == requirement.id
                }))
                && !edges.iter().any(|edge| {
                    edge.scope_id == *scope
                        && edge.edge_type == EdgeType::Produces
                        && edge.from_type == NodeType::Requirement
                        && edge.from_id == requirement.id
                        && edge.to_type == NodeType::Rule
                })
        })
        .map(|requirement| GapItem {
            requirement_id: requirement.id.as_str().to_string(),
            reason: "resolved requirement has no downstream rule".to_string(),
        })
        .collect())
}
