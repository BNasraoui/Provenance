use crate::wiki::model::{CorpusCounts, IndexEntry, OrphanReport, ScopeIndexPage};
use provenance_core::{EdgeType, NodeType};

use super::super::context::Assembler;
use super::super::page_links::{requirement_link, resolution_link, rule_link, source_link};
use provenance_store::cache::GapKind;

impl Assembler<'_> {
    #[allow(clippy::too_many_lines)]
    pub(in crate::wiki::assemble) fn index_page(&self) -> ScopeIndexPage {
        let roots: Vec<IndexEntry> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| !self.has_parent_edge(&requirement.id))
            .map(|requirement| IndexEntry {
                link: requirement_link(requirement),
                status: requirement.status.clone(),
                children: self
                    .edges()
                    .filter(|edge| {
                        edge.edge_type == EdgeType::RefinesInto
                            && edge.from_type == NodeType::Requirement
                            && edge.from_id == requirement.id
                    })
                    .count(),
                resolutions: self.resolving_resolutions(&requirement.id).len(),
                rules: self.produced_rules_for_requirement(&requirement.id).len(),
            })
            .collect();
        let orphans = OrphanReport {
            rules: self
                .gaps
                .iter()
                .filter(|gap| gap.kind == GapKind::OrphanRule)
                .filter_map(|gap| {
                    self.state
                        .rules
                        .iter()
                        .find(|rule| rule.id.as_str() == gap.node_id)
                })
                .map(rule_link)
                .collect(),
            resolutions: self
                .gaps
                .iter()
                .filter(|gap| gap.kind == GapKind::OrphanResolution)
                .filter_map(|gap| {
                    self.state
                        .resolutions
                        .iter()
                        .find(|resolution| resolution.id.as_str() == gap.node_id)
                })
                .map(resolution_link)
                .collect(),
            sources: self
                .gaps
                .iter()
                .filter(|gap| gap.kind == GapKind::UnreferencedSource)
                .filter_map(|gap| {
                    self.state
                        .sources
                        .iter()
                        .find(|source| source.id.as_str() == gap.node_id)
                })
                .map(source_link)
                .collect(),
        };
        ScopeIndexPage {
            scope: self.state.scope.clone(),
            title: self.state.scope.clone(),
            counts: CorpusCounts {
                sources: self.state.sources.len(),
                requirements: self.state.requirements.len(),
                resolutions: self.state.resolutions.len(),
                rules: self.state.rules.len(),
            },
            roots,
            gaps: self.index_gaps(),
            orphans,
        }
    }
}
