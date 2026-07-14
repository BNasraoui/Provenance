mod contradiction;
mod dangling;
mod frontier;
mod graph_query;
mod model;
mod state_adapter;

pub use graph_query::GapGraph;
pub use model::{node_type_word, GapItem, GapKind};
pub use state_adapter::find_gaps;

use graph_query::GraphQuery;

/// Computes gaps in stable policy order. This is also the seam for callers
/// that already hold graph records rather than a state store.
pub fn compute_gaps(graph: &GapGraph<'_>) -> Vec<GapItem> {
    let query = GraphQuery::new(graph);
    let mut gaps = Vec::new();
    frontier::add_requirement_gaps(&query, &mut gaps);
    frontier::add_resolution_gaps(&query, &mut gaps);
    frontier::add_rule_gaps(&query, &mut gaps);
    frontier::add_source_gaps(&query, &mut gaps);
    dangling::add_reference_gaps(&query, &mut gaps);
    contradiction::add_gaps(&query, &mut gaps);
    frontier::add_question_gaps(&query, &mut gaps);
    frontier::add_topic_gaps(&query, &mut gaps);
    gaps
}

#[cfg(test)]
mod tests;
