use crate::wiki::model::{PageId, PageKind, PageLink, SourcePage};
use provenance_core::{EdgeType, NodeType, Source};

use super::super::context::Assembler;
use super::super::page_links::{requirement_link, source_link};

impl<'a> Assembler<'a> {
    pub(in crate::wiki::assemble) fn source_page(&self, source: &'a Source) -> SourcePage {
        let referenced_requirements: Vec<PageLink> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| {
                self.edge_exists(
                    EdgeType::References,
                    NodeType::Source,
                    &source.id,
                    NodeType::Requirement,
                    &requirement.id,
                ) || requirement
                    .source_refs
                    .iter()
                    .any(|reference| reference.source_id == source.id)
            })
            .map(requirement_link)
            .collect();
        let superseded_by = source
            .superseded_by
            .as_ref()
            .and_then(|id| self.find_source(id).map(source_link));
        SourcePage {
            id: PageId::new(PageKind::Source, source.id.as_str()),
            title: source.name.clone(),
            source_type: source.source_type.clone(),
            url: source.url.clone(),
            reference: self.source_reference_link(source),
            commit_pin: source.commit_pin.clone(),
            effective_date: source.effective_date,
            review_date: source.review_date,
            superseded_by,
            referenced_requirements,
            gaps: self.gaps_for(NodeType::Source, &source.id),
            threads: self.threads_for(NodeType::Source, &source.id),
        }
    }
}
