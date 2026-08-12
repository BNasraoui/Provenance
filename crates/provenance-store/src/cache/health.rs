use crate::cache::find_gaps;
use crate::cache::gaps::{GraphQuery, GraphRecords};
use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{EdgeType, NodeType, RequirementStatus};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEvidenceReference {
    pub subject_id: String,
    pub document: String,
    pub section: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEvidence {
    pub rule_ids: BTreeSet<String>,
    pub references: Vec<GraphEvidenceReference>,
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
    pub gaps: CountMetric,
}

#[derive(Debug, serde::Serialize)]
pub struct OrphanRuleItem {
    pub rule_id: String,
    pub missing: Vec<String>,
}

/// Loads every canonical graph reference that names a repository path.
///
/// Rule source documents are direct citations. A Source becomes a citation
/// only when a Requirement names it through an embedded source reference or a
/// typed `references` edge; unreferenced catalog entries are not evidence for
/// anything in the graph. Path syntax is interpreted by the command layer.
pub fn graph_evidence(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<GraphEvidence> {
    let store = StateStore::new(layout.clone());
    store.with_repository_publication(|| graph_evidence_locked(scope, &store))
}

fn graph_evidence_locked(
    scope: &provenance_core::ScopeId,
    store: &StateStore,
) -> anyhow::Result<GraphEvidence> {
    let edges = store.list_edges()?;
    let requirements = store.list_requirements(scope)?;
    let mut cited_sources = requirements
        .iter()
        .flat_map(|requirement| &requirement.source_refs)
        .map(|reference| reference.source_id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    cited_sources.extend(
        edges
            .iter()
            .filter(|edge| {
                edge.scope_id == *scope
                    && edge.edge_type == EdgeType::References
                    && edge.from_type == NodeType::Source
                    && edge.to_type == NodeType::Requirement
            })
            .map(|edge| edge.from_id.as_str().to_string()),
    );
    let rules = store.list_rules(scope)?;
    let rule_ids = rules
        .iter()
        .map(|rule| rule.id.as_str().to_string())
        .collect();
    let mut references = rules
        .into_iter()
        .filter_map(|rule| {
            rule.source_document.map(|document| GraphEvidenceReference {
                subject_id: rule.id.as_str().to_string(),
                document,
                section: rule.source_section,
            })
        })
        .collect::<Vec<_>>();
    references.extend(store.list_sources(scope)?.into_iter().filter_map(|source| {
        (cited_sources.contains(source.id.as_str()))
            .then_some(source.reference)
            .flatten()
            .map(|document| GraphEvidenceReference {
                subject_id: source.id.as_str().to_string(),
                document,
                section: None,
            })
    }));
    references.sort_by(|left, right| {
        (&left.subject_id, &left.document, &left.section).cmp(&(
            &right.subject_id,
            &right.document,
            &right.section,
        ))
    });
    references.dedup();
    Ok(GraphEvidence {
        rule_ids,
        references,
    })
}

pub fn coverage_health(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<HealthView> {
    let store = StateStore::new(layout.clone());
    store.with_repository_publication(|| coverage_health_locked(layout, scope, &store))
}

fn coverage_health_locked(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    store: &StateStore,
) -> anyhow::Result<HealthView> {
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
        gaps: CountMetric { total: gaps },
    })
}

/// Rules whose trace back to a source is incomplete.
///
/// A rule is complete only when a requirement produces it and a source
/// reaches that requirement. A resolution may also produce the rule, but is
/// not required. The producer test is the same join the `OrphanRule` gap
/// runs, so `orphans` and `gaps` name the same rules; `orphans` additionally
/// reports the rules whose producing requirement has no live source behind
/// it.
pub fn orphan_rules(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<Vec<OrphanRuleItem>> {
    let store = StateStore::new(layout.clone());
    store.with_repository_publication(|| orphan_rules_locked(scope, &store))
}

fn orphan_rules_locked(
    scope: &provenance_core::ScopeId,
    store: &StateStore,
) -> anyhow::Result<Vec<OrphanRuleItem>> {
    let records = GraphRecords::load(scope, store)?;
    let graph = records.graph(scope);
    let query = GraphQuery::new(&graph);
    Ok(records
        .rules
        .iter()
        .filter_map(|rule| {
            let mut missing: Vec<String> = query
                .missing_rule_producers(&rule.id)
                .into_iter()
                .map(|producer| producer.word().to_string())
                .collect();
            if !query.rule_trace_reaches_source(&rule.id) {
                missing.push("source".to_string());
            }
            (!missing.is_empty()).then(|| OrphanRuleItem {
                rule_id: rule.id.as_str().to_string(),
                missing,
            })
        })
        .collect())
}
