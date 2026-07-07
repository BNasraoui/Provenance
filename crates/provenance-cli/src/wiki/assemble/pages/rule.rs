use crate::wiki::model::{PageId, PageKind, PageLink, RulePage};
use provenance_core::{EdgeType, NodeType, Requirement, Resolution, Rule};
use std::collections::BTreeSet;

use super::super::context::Assembler;
use super::super::page_links::{requirement_link, resolution_link, rule_title, source_link};

impl<'a> Assembler<'a> {
    #[allow(clippy::too_many_lines)]
    pub(in crate::wiki::assemble) fn rule_page(&self, rule: &'a Rule) -> RulePage {
        let producing_resolutions: Vec<&Resolution> = self
            .state
            .resolutions
            .iter()
            .filter(|resolution| {
                self.edge_exists(
                    EdgeType::Produces,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Rule,
                    &rule.id,
                )
            })
            .collect();
        let producing_requirements: Vec<&Requirement> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| {
                self.edge_exists(
                    EdgeType::Produces,
                    NodeType::Requirement,
                    &requirement.id,
                    NodeType::Rule,
                    &rule.id,
                )
            })
            .collect();
        let produced_by: Vec<PageLink> = producing_resolutions
            .iter()
            .copied()
            .map(resolution_link)
            .chain(producing_requirements.iter().copied().map(requirement_link))
            .collect();
        let mut requirement_ids: BTreeSet<&str> = producing_requirements
            .iter()
            .map(|requirement| requirement.id.as_str())
            .collect();
        for resolution in &producing_resolutions {
            for requirement in &self.state.requirements {
                if self.edge_exists(
                    EdgeType::Resolves,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Requirement,
                    &requirement.id,
                ) {
                    requirement_ids.insert(requirement.id.as_str());
                }
            }
        }
        let upstream_requirements: Vec<&Requirement> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| requirement_ids.contains(requirement.id.as_str()))
            .collect();
        let sources: Vec<PageLink> = self
            .state
            .sources
            .iter()
            .filter(|source| {
                upstream_requirements.iter().any(|requirement| {
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
            })
            .map(source_link)
            .collect();
        RulePage {
            id: PageId::new(PageKind::Rule, rule.id.as_str()),
            title: rule_title(rule),
            rule_code: rule.rule_code.clone(),
            statement: rule.statement.clone(),
            description: rule.description.clone(),
            status: rule.status.clone(),
            severity: rule.severity.clone(),
            modality: rule.modality.clone(),
            rule_type: rule.rule_type.clone(),
            confidence: rule.confidence,
            extraction_method: rule.extraction_method.clone(),
            source_document: rule.source_document.clone(),
            source_section: rule.source_section.clone(),
            evidence: self.rule_evidence(rule),
            produced_by,
            requirements: upstream_requirements
                .into_iter()
                .map(requirement_link)
                .collect(),
            sources,
            gaps: self.gaps_for(NodeType::Rule, &rule.id),
            threads: self.threads_for(NodeType::Rule, &rule.id),
        }
    }
}
