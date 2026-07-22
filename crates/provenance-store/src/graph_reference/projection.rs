use super::GraphReferenceError;
use crate::{layout::ProvenanceLayout, state_store::StateStore};
use camino::Utf8Path;
use provenance_core::{
    Boundary, Domain, Edge, Question, Requirement, Resolution, Rule, Scope, ScopeId, Service,
    ServiceBinding, Source, Topic,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphExport {
    pub schema_version: u32,
    pub scope: Scope,
    pub sources: Vec<Source>,
    pub domains: Vec<Domain>,
    pub requirements: Vec<Requirement>,
    pub boundaries: Vec<Boundary>,
    pub topics: Vec<Topic>,
    pub questions: Vec<Question>,
    pub resolutions: Vec<Resolution>,
    pub rules: Vec<Rule>,
    pub services: Vec<Service>,
    pub service_bindings: Vec<ServiceBinding>,
    pub edges: Vec<Edge>,
}

pub(super) fn load_projection(
    repository: &Utf8Path,
    scope: &str,
) -> Result<GraphExport, GraphReferenceError> {
    let scope_id = ScopeId::new(scope.to_string()).map_err(incomplete)?;
    let store = StateStore::new(ProvenanceLayout::new(repository.to_path_buf()));
    let manifest = store.manifest().map_err(|error| {
        let detail = error.to_string();
        if repository.join(".provenance/state/manifest.json").exists() {
            incomplete(detail)
        } else {
            GraphReferenceError::Missing {
                detail: "canonical manifest is absent".into(),
            }
        }
    })?;
    if manifest.schema_version.0 != 1 {
        return Err(incomplete(format!(
            "unsupported manifest schema_version {}",
            manifest.schema_version.0
        )));
    }
    let selected_scope = manifest
        .scopes
        .into_iter()
        .find(|candidate| candidate.id == scope_id)
        .ok_or_else(|| GraphReferenceError::Missing {
            detail: format!("scope '{scope}' is absent from the pinned manifest"),
        })?;

    let mut graph = GraphExport {
        schema_version: 1,
        scope: selected_scope,
        sources: store.list_sources(&scope_id).map_err(incomplete)?,
        domains: store.list_domains(&scope_id).map_err(incomplete)?,
        requirements: store.list_requirements(&scope_id).map_err(incomplete)?,
        boundaries: store.list_boundaries(&scope_id).map_err(incomplete)?,
        topics: store.list_topics(&scope_id).map_err(incomplete)?,
        questions: store.list_questions(&scope_id).map_err(incomplete)?,
        resolutions: store.list_resolutions(&scope_id).map_err(incomplete)?,
        rules: store.list_rules(&scope_id).map_err(incomplete)?,
        services: store.list_services(&scope_id).map_err(incomplete)?,
        service_bindings: store.list_service_bindings(&scope_id).map_err(incomplete)?,
        edges: store
            .list_edges()
            .map_err(incomplete)?
            .into_iter()
            .filter(|edge| edge.scope_id == scope_id)
            .collect(),
    };
    validate_scope_ownership(&graph, &scope_id)?;
    sort_records(&mut graph);
    Ok(graph)
}

fn sort_records(graph: &mut GraphExport) {
    graph
        .sources
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .domains
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .requirements
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .boundaries
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .topics
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .questions
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .resolutions
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph.rules.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .services
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph
        .service_bindings
        .sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    graph.edges.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
}

fn validate_scope_ownership(
    graph: &GraphExport,
    scope: &ScopeId,
) -> Result<(), GraphReferenceError> {
    macro_rules! require_scope {
        ($records:expr, $kind:literal) => {
            for record in $records {
                if &record.scope_id != scope {
                    return Err(GraphReferenceError::Incomplete {
                        detail: format!(
                            "{} '{}' belongs to scope '{}', not '{}'",
                            $kind,
                            record.id.as_str(),
                            record.scope_id.as_str(),
                            scope.as_str()
                        ),
                    });
                }
            }
        };
    }
    require_scope!(&graph.sources, "source");
    require_scope!(&graph.domains, "domain");
    require_scope!(&graph.requirements, "requirement");
    require_scope!(&graph.boundaries, "boundary");
    require_scope!(&graph.topics, "topic");
    require_scope!(&graph.questions, "question");
    require_scope!(&graph.resolutions, "resolution");
    require_scope!(&graph.rules, "rule");
    require_scope!(&graph.services, "service");
    require_scope!(&graph.service_bindings, "service binding");
    require_scope!(&graph.edges, "edge");
    Ok(())
}

fn incomplete(error: impl std::fmt::Display) -> GraphReferenceError {
    GraphReferenceError::Incomplete {
        detail: error.to_string(),
    }
}
