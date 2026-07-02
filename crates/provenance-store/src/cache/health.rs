use crate::cache::find_gaps;
use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{EdgeType, NodeType, RequirementStatus, ResolutionStatus};

#[derive(Debug, serde::Serialize)]
pub struct StaleItem {
    pub resolution_id: String,
    pub reason: String,
}

#[derive(Debug, serde::Serialize)]
pub struct CountMetric {
    pub total: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct RuleHealthMetric {
    pub total: usize,
    pub with_complete_traceability: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct HealthView {
    pub requirements: CountMetric,
    pub source_linked_requirements: usize,
    pub resolved_requirements: usize,
    pub requirements_with_rules: usize,
    pub rules: RuleHealthMetric,
    pub stale: CountMetric,
    pub gaps: CountMetric,
}

#[derive(Debug, serde::Serialize)]
pub struct OrphanRuleItem {
    pub rule_id: String,
    pub missing: Vec<String>,
}

pub fn find_stale(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<Vec<StaleItem>> {
    let store = StateStore::new(layout.clone());
    let edges = store.list_edges()?;
    Ok(store
        .list_resolutions(scope)?
        .into_iter()
        .filter_map(|resolution| {
            if resolution.status == ResolutionStatus::Approved
                && resolution
                    .review_on
                    .as_deref()
                    .is_some_and(|date| date < "2099-01-01")
            {
                Some(StaleItem {
                    resolution_id: resolution.id.as_str().to_string(),
                    reason: "approved resolution is past review date".to_string(),
                })
            } else if edges.iter().any(|edge| {
                edge.scope_id == *scope
                    && edge.edge_type == EdgeType::Supersedes
                    && edge.to_id == resolution.id
            }) {
                Some(StaleItem {
                    resolution_id: resolution.id.as_str().to_string(),
                    reason: "upstream source was superseded".to_string(),
                })
            } else {
                None
            }
        })
        .collect())
}

pub fn coverage_health(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<HealthView> {
    let store = StateStore::new(layout.clone());
    let requirements = store.list_requirements(scope)?;
    let rules = store.list_rules(scope)?;
    let edges: Vec<_> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| edge.scope_id == *scope)
        .collect();
    let source_linked_requirements = requirements
        .iter()
        .filter(|req| {
            edges
                .iter()
                .any(|edge| edge.edge_type == EdgeType::References && edge.to_id == req.id)
        })
        .count();
    let resolved_requirements = requirements
        .iter()
        .filter(|req| {
            req.status == RequirementStatus::Resolved
                || edges
                    .iter()
                    .any(|edge| edge.edge_type == EdgeType::Resolves && edge.to_id == req.id)
        })
        .count();
    let requirements_with_rules = requirements
        .iter()
        .filter(|req| {
            edges.iter().any(|edge| {
                edge.edge_type == EdgeType::Produces
                    && edge.from_type == NodeType::Requirement
                    && edge.from_id == req.id
            })
        })
        .count();
    let orphan_count = orphan_rules(layout, scope)?.len();
    let with_complete_traceability = rules.len().saturating_sub(orphan_count);
    let stale = find_stale(layout, scope)?.len();
    let gaps = find_gaps(layout, scope)?.len();
    Ok(HealthView {
        requirements: CountMetric {
            total: requirements.len(),
        },
        source_linked_requirements,
        resolved_requirements,
        requirements_with_rules,
        rules: RuleHealthMetric {
            total: rules.len(),
            with_complete_traceability,
        },
        stale: CountMetric { total: stale },
        gaps: CountMetric { total: gaps },
    })
}

pub fn orphan_rules(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<Vec<OrphanRuleItem>> {
    let store = StateStore::new(layout.clone());
    let edges: Vec<_> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| edge.scope_id == *scope)
        .collect();
    Ok(store
        .list_rules(scope)?
        .into_iter()
        .filter_map(|rule| {
            let has_requirement = edges.iter().any(|edge| {
                edge.edge_type == EdgeType::Produces
                    && edge.to_id == rule.id
                    && edge.from_type == NodeType::Requirement
            });
            let has_resolution = edges.iter().any(|edge| {
                edge.edge_type == EdgeType::Produces
                    && edge.to_id == rule.id
                    && edge.from_type == NodeType::Resolution
            });
            let has_source = has_requirement
                && edges.iter().any(|edge| {
                    edge.edge_type == EdgeType::References && edge.to_type == NodeType::Requirement
                });
            let mut missing = Vec::new();
            if !has_requirement {
                missing.push("requirement".to_string());
            }
            if !has_resolution {
                missing.push("resolution".to_string());
            }
            if !has_source {
                missing.push("source".to_string());
            }
            (!missing.is_empty()).then(|| OrphanRuleItem {
                rule_id: rule.id.as_str().to_string(),
                missing,
            })
        })
        .collect())
}
