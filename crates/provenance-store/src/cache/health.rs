use crate::cache::find_gaps;
use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{
    EdgeType, NodeType, RequirementStatus, ResolutionStatus, Rule, RuleSeverity,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, serde::Serialize)]
pub struct StaleItem {
    pub resolution_id: String,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct StaleOptions {
    pub min_age_days: u32,
    pub rule_severities: Vec<RuleSeverity>,
    pub min_downstream_rules: u32,
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
    find_stale_with_options(layout, scope, &StaleOptions::default())
}

pub fn find_stale_with_options(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    options: &StaleOptions,
) -> anyhow::Result<Vec<StaleItem>> {
    let store = StateStore::new(layout.clone());
    let edges = store.list_edges()?;
    let rules = store.list_rules(scope)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(store
        .list_resolutions(scope)?
        .into_iter()
        .filter(|resolution| {
            resolution_is_old_enough(resolution.approved_at, options.min_age_days, now)
        })
        .filter(|resolution| {
            let downstream_rules = rules
                .iter()
                .filter(|rule| {
                    is_downstream_rule(&edges, scope, resolution.id.as_str(), rule)
                        && (options.rule_severities.is_empty()
                            || options.rule_severities.contains(&rule.severity))
                })
                .count();
            let required_rules = if options.rule_severities.is_empty() {
                options.min_downstream_rules
            } else {
                options.min_downstream_rules.max(1)
            };
            downstream_rules >= required_rules as usize
        })
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

fn resolution_is_old_enough(approved_at: Option<i64>, min_age_days: u32, now: u64) -> bool {
    if min_age_days == 0 {
        return true;
    }
    let Some(mut approved_at) = approved_at.and_then(|timestamp| u64::try_from(timestamp).ok())
    else {
        return false;
    };
    // Timestamps in existing state may be seconds or milliseconds since the Unix epoch.
    if approved_at >= 100_000_000_000 {
        approved_at /= 1_000;
    }
    now.saturating_sub(approved_at) / 86_400 >= u64::from(min_age_days)
}

fn is_downstream_rule(
    edges: &[provenance_core::Edge],
    scope: &provenance_core::ScopeId,
    resolution_id: &str,
    rule: &Rule,
) -> bool {
    edges.iter().any(|edge| {
        edge.scope_id == *scope
            && edge.edge_type == EdgeType::Produces
            && edge.from_type == NodeType::Resolution
            && edge.from_id.as_str() == resolution_id
            && edge.to_type == NodeType::Rule
            && edge.to_id == rule.id
    })
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
