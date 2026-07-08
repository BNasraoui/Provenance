use crate::wiki::model::{LineageEntry, PageLink};
use provenance_core::{EdgeType, NodeType, Requirement, Resolution, Rule, StableId};
use std::collections::BTreeSet;

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

    pub(super) fn has_parent_edge(&self, requirement_id: &StableId) -> bool {
        self.edges().any(|edge| {
            edge.edge_type == EdgeType::RefinesInto
                && edge.from_type == NodeType::Requirement
                && edge.to_type == NodeType::Requirement
                && edge.to_id == *requirement_id
        })
    }

    pub(super) fn resolving_resolutions(&self, requirement_id: &StableId) -> Vec<&'a Resolution> {
        self.state
            .resolutions
            .iter()
            .filter(|resolution| {
                self.edge_exists(
                    EdgeType::Resolves,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Requirement,
                    requirement_id,
                )
            })
            .collect()
    }
    pub(super) fn produced_rules_for_requirement(
        &self,
        requirement_id: &StableId,
    ) -> Vec<&'a Rule> {
        let resolution_ids: BTreeSet<&str> = self
            .resolving_resolutions(requirement_id)
            .into_iter()
            .map(|resolution| resolution.id.as_str())
            .collect();
        self.state
            .rules
            .iter()
            .filter(|rule| {
                self.edges().any(|edge| {
                    edge.edge_type == EdgeType::Produces
                        && edge.to_type == NodeType::Rule
                        && edge.to_id == rule.id
                        && ((edge.from_type == NodeType::Requirement
                            && edge.from_id == *requirement_id)
                            || (edge.from_type == NodeType::Resolution
                                && resolution_ids.contains(edge.from_id.as_str())))
                })
            })
            .collect()
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

    pub(super) fn produced_rules_for_resolution(&self, resolution_id: &StableId) -> Vec<&'a Rule> {
        self.state
            .rules
            .iter()
            .filter(|rule| {
                self.edge_exists(
                    EdgeType::Produces,
                    NodeType::Resolution,
                    resolution_id,
                    NodeType::Rule,
                    &rule.id,
                )
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
