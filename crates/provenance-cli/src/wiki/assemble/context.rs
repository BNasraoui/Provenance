use crate::handlers::ScopeExport;
use crate::wiki::links::LinkResolver;
use provenance_core::{Edge, EdgeType, NodeType, Requirement, Source, StableId};
use provenance_store::cache::GapItem;

pub(super) struct Assembler<'a> {
    pub(super) state: &'a ScopeExport,
    pub(super) resolver: &'a LinkResolver,
    pub(super) gaps: &'a [GapItem],
}

impl<'a> Assembler<'a> {
    pub(super) fn edges(&self) -> impl Iterator<Item = &'a Edge> {
        let scope = self.state.scope.as_str();
        self.state
            .edges
            .iter()
            .filter(move |edge| edge.scope_id.as_str() == scope)
    }

    /// Matches an edge by its full identity: type, kind, and id on both
    /// ends. Stable ids are not namespaced per record kind, so matching on
    /// `from_id`/`to_id` alone would let a `Source` and a `Resolution` that
    /// happen to share an id get cross-wired; `from_type`/`to_type` must be
    /// checked too.
    pub(super) fn edge_exists(
        &self,
        edge_type: EdgeType,
        from_type: NodeType,
        from_id: &StableId,
        to_type: NodeType,
        to_id: &StableId,
    ) -> bool {
        self.edges().any(|edge| {
            edge.edge_type == edge_type
                && edge.from_type == from_type
                && edge.from_id == *from_id
                && edge.to_type == to_type
                && edge.to_id == *to_id
        })
    }

    pub(super) fn find_requirement(&self, id: &StableId) -> Option<&'a Requirement> {
        self.state
            .requirements
            .iter()
            .find(|requirement| requirement.id == *id)
    }

    pub(super) fn find_source(&self, id: &StableId) -> Option<&'a Source> {
        self.state.sources.iter().find(|source| source.id == *id)
    }
}
