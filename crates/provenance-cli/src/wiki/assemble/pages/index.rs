use crate::wiki::model::{
    CorpusCounts, DomainIndexPage, DomainState, HomepageDomain, IndexEntry, OrphanRecord,
    OrphanReport, ScopeIndexPage, SearchIndexPage, HOMEPAGE_DOMAIN_ROW_CAP,
};
use provenance_core::{EdgeType, NodeType};

use super::super::context::Assembler;
use super::super::page_links::{
    display_identifier, reader_title, requirement_link, resolution_link, rule_link, source_link,
};
use provenance_store::cache::GapKind;

impl Assembler<'_> {
    #[allow(clippy::too_many_lines)]
    pub(in crate::wiki::assemble) fn index_page(
        &self,
        domains: &DomainIndexPage,
        search: &SearchIndexPage,
        unfinished_count: usize,
    ) -> ScopeIndexPage {
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
                        .map(|rule| OrphanRecord {
                            link: rule_link(rule),
                            reason: gap.reason.clone(),
                        })
                })
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
                        .map(|resolution| OrphanRecord {
                            link: resolution_link(resolution),
                            reason: gap.reason.clone(),
                        })
                })
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
                        .map(|source| OrphanRecord {
                            link: source_link(source),
                            reason: gap.reason.clone(),
                        })
                })
                .collect(),
        };
        let authored_domain_count = domains
            .groups
            .iter()
            .filter(|group| matches!(group.state, DomainState::Defined { .. }))
            .count();
        let domains = domains
            .groups
            .iter()
            .filter_map(|group| match &group.state {
                DomainState::Defined {
                    id,
                    name,
                    description,
                } => Some(HomepageDomain {
                    id: id.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    requirements: group.requirements.len(),
                    rules: group.rules.len(),
                }),
                DomainState::Missing { .. } | DomainState::Unassigned => None,
            })
            .take(HOMEPAGE_DOMAIN_ROW_CAP)
            .collect();
        ScopeIndexPage {
            scope: self.state.scope.clone(),
            title: reader_title(&format!(
                "{} documentation",
                display_identifier(&self.state.scope)
            )),
            counts: CorpusCounts {
                sources: self.state.sources.len(),
                requirements: self.state.requirements.len(),
                resolutions: self.state.resolutions.len(),
                rules: self.state.rules.len(),
            },
            roots,
            gaps: self.index_gaps(),
            orphans,
            search_coverage: search.coverage.clone(),
            search_example: search.example.clone(),
            domains,
            authored_domain_count,
            unfinished_count,
        }
    }
}
