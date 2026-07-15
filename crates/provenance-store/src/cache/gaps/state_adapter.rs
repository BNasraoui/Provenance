use super::{compute_gaps, graph_query::GapGraph, model::GapItem};
use crate::{
    layout::ProvenanceLayout,
    state_store::{ScopeSnapshot, StateStore},
};
use provenance_core::ScopeId;

pub fn find_gaps(layout: &ProvenanceLayout, scope: &ScopeId) -> anyhow::Result<Vec<GapItem>> {
    let snapshot = StateStore::new(layout.clone()).scope_snapshot(scope)?;
    Ok(find_gaps_in_snapshot(&snapshot))
}

pub(in crate::cache) fn find_gaps_in_snapshot(snapshot: &ScopeSnapshot) -> Vec<GapItem> {
    compute_gaps(&GapGraph {
        scope: &snapshot.scope,
        sources: &snapshot.sources,
        requirements: &snapshot.requirements,
        resolutions: &snapshot.resolutions,
        rules: &snapshot.rules,
        topics: &snapshot.topics,
        questions: &snapshot.questions,
        edges: &snapshot.edges,
        threads: &snapshot.threads,
    })
}
