use crate::wiki::model::{
    CorpusCounts, DomainIndexPage, DomainState, HomepageDomain, ScopeIndexPage, SearchIndexPage,
    HOMEPAGE_DOMAIN_ROW_CAP,
};

use super::super::context::Assembler;
use super::super::page_links::{display_identifier, reader_title};

impl Assembler<'_> {
    pub(in crate::wiki::assemble) fn index_page(
        &self,
        domains: &DomainIndexPage,
        search: &SearchIndexPage,
        unfinished_count: usize,
    ) -> ScopeIndexPage {
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
            search_coverage: search.coverage.clone(),
            search_example: search.example.clone(),
            domains,
            authored_domain_count,
            unfinished_count,
        }
    }
}
