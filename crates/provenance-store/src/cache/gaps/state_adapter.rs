use super::{compute_gaps, GapGraph, GapItem};
use crate::{layout::ProvenanceLayout, state_store::StateStore};
use provenance_core::ScopeId;

pub fn find_gaps(layout: &ProvenanceLayout, scope: &ScopeId) -> anyhow::Result<Vec<GapItem>> {
    let store = StateStore::new(layout.clone());
    let edges = store.list_edges()?;
    let sources = store.list_sources(scope)?;
    let requirements = store.list_requirements(scope)?;
    let resolutions = store.list_resolutions(scope)?;
    let rules = store.list_rules(scope)?;
    let topics = store.list_topics(scope)?;
    let questions = store.list_questions(scope)?;
    let threads = store.list_threads(scope)?;
    Ok(compute_gaps(&GapGraph {
        scope,
        sources: &sources,
        requirements: &requirements,
        resolutions: &resolutions,
        rules: &rules,
        topics: &topics,
        questions: &questions,
        edges: &edges,
        threads: &threads,
    }))
}
