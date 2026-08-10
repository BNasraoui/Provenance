use crate::wiki::model::{LineageEntry, PageLink};
use provenance_core::{EdgeType, NodeType, Requirement, Resolution, Rule, StableId};
use std::collections::{BTreeMap, BTreeSet};

use super::context::Assembler;
use super::page_links::requirement_link;

impl<'a> Assembler<'a> {
    pub(super) fn parent_ids_of(&self, requirement_id: &StableId) -> Vec<&'a StableId> {
        let mut parent_ids: Vec<&StableId> = self
            .edges()
            .filter(|edge| {
                edge.edge_type == EdgeType::RefinesInto
                    && edge.from_type == NodeType::Requirement
                    && edge.to_type == NodeType::Requirement
                    && edge.to_id == *requirement_id
            })
            .map(|edge| &edge.from_id)
            .collect();
        parent_ids.sort_by_key(|id| id.as_str());
        parent_ids
    }

    pub(super) fn parent_of(&self, requirement_id: &StableId) -> Option<&'a Requirement> {
        let parent_ids = self.parent_ids_of(requirement_id);
        parent_ids
            .into_iter()
            .find_map(|id| self.find_requirement(id))
    }

    pub(super) fn resolving_resolutions(&self, requirement_id: &StableId) -> Vec<&'a Resolution> {
        self.query.resolving_resolutions(requirement_id)
    }

    pub(super) fn produced_rules_for_requirement(
        &self,
        requirement_id: &StableId,
    ) -> Vec<&'a Rule> {
        self.query.produced_rules_for_requirement(requirement_id)
    }

    pub(super) fn produced_rules_for_resolution(&self, resolution_id: &StableId) -> Vec<&'a Rule> {
        self.query.produced_rules_for_resolution(resolution_id)
    }

    /// The requirements a rule answers to, in record order.
    ///
    /// This is the inverse of [`Self::produced_rules_for_requirement`], read
    /// off one pass of that forward traversal rather than walking `Produces`
    /// and `Resolves` backwards a second time. A requirement page listing a
    /// rule and that rule's page listing the requirement are then the same
    /// fact, not two facts that happen to agree.
    pub(super) fn requirements_behind_rule(&self, rule_id: &StableId) -> &[&'a Requirement] {
        self.rule_requirements
            .get_or_init(|| {
                let mut attribution: BTreeMap<&'a str, Vec<&'a Requirement>> = BTreeMap::new();
                for requirement in &self.state.requirements {
                    for rule in self.produced_rules_for_requirement(&requirement.id) {
                        let attributed = attribution.entry(rule.id.as_str()).or_default();
                        // Two rule records sharing an id would otherwise list
                        // the same requirement twice; the outer loop visits
                        // each requirement once, so checking the tail is enough.
                        if attributed
                            .last()
                            .is_none_or(|last| last.id != requirement.id)
                        {
                            attributed.push(requirement);
                        }
                    }
                }
                attribution
            })
            .get(rule_id.as_str())
            .map_or(&[], Vec::as_slice)
    }

    pub(super) fn sibling_requirements(&self, requirement_id: &StableId) -> Vec<PageLink> {
        let parent_ids: BTreeSet<&str> = self
            .parent_ids_of(requirement_id)
            .into_iter()
            .map(StableId::as_str)
            .collect();
        if parent_ids.is_empty() {
            return Vec::new();
        }

        self.state
            .requirements
            .iter()
            .filter_map(|candidate| {
                if candidate.id == *requirement_id {
                    return None;
                }
                let has_shared_parent = self.edges().any(|edge| {
                    edge.edge_type == EdgeType::RefinesInto
                        && edge.from_type == NodeType::Requirement
                        && parent_ids.contains(edge.from_id.as_str())
                        && edge.to_type == NodeType::Requirement
                        && edge.to_id == candidate.id
                });
                if has_shared_parent {
                    Some(requirement_link(candidate))
                } else {
                    None
                }
            })
            .collect()
    }

    pub(super) fn lineage(&self, requirement: &'a Requirement) -> Vec<LineageEntry> {
        let mut chain = vec![requirement];
        let mut visited: BTreeSet<&str> = BTreeSet::from([requirement.id.as_str()]);
        let mut current = requirement;
        while let Some(parent) = self.parent_of(&current.id) {
            if !visited.insert(parent.id.as_str()) {
                break;
            }
            chain.push(parent);
            current = parent;
        }
        chain.reverse();
        let last = chain.len() - 1;
        chain
            .into_iter()
            .enumerate()
            .map(|(index, entry)| LineageEntry {
                link: requirement_link(entry),
                is_current: index == last,
            })
            .collect()
    }
}
