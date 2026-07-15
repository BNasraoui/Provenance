use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{EdgeType, NodeType, RequirementStatus};

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

pub fn coverage_health(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<HealthView> {
    let snapshot = StateStore::new(layout.clone()).scope_snapshot(scope)?;
    let requirements = &snapshot.requirements;
    let rules = &snapshot.rules;
    let edges = &snapshot.edges;
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
    let orphan_count = orphan_rules_in_snapshot(&snapshot).len();
    let with_complete_traceability = rules.len().saturating_sub(orphan_count);
    let stale = crate::cache::stale::find_stale_in_current_snapshot(&snapshot)?.len();
    let gaps = crate::cache::gaps::find_gaps_in_snapshot(&snapshot).len();
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
    let snapshot = StateStore::new(layout.clone()).scope_snapshot(scope)?;
    Ok(orphan_rules_in_snapshot(&snapshot))
}

fn orphan_rules_in_snapshot(snapshot: &crate::state_store::ScopeSnapshot) -> Vec<OrphanRuleItem> {
    let edges = &snapshot.edges;
    snapshot
        .rules
        .iter()
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
        .collect()
}
