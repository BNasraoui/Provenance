use provenance_core::{Edge, EdgeType, NodeType, Resolution, Rule, RuleSeverity, StableId};
use std::collections::BTreeSet;

/// Canonical graph query for rules produced downstream of requirements/resolutions.
pub struct DownstreamRuleQuery<'a> {
    edges: &'a [Edge],
    resolutions: &'a [Resolution],
    rules: &'a [Rule],
}

impl<'a> DownstreamRuleQuery<'a> {
    pub const fn new(edges: &'a [Edge], resolutions: &'a [Resolution], rules: &'a [Rule]) -> Self {
        Self {
            edges,
            resolutions,
            rules,
        }
    }

    pub fn for_requirement(&self, requirement_id: &StableId) -> Vec<&'a Rule> {
        let resolutions: BTreeSet<_> = self
            .resolutions
            .iter()
            .filter(|resolution| {
                self.has_edge(
                    EdgeType::Resolves,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Requirement,
                    requirement_id,
                )
            })
            .map(|resolution| resolution.id.as_str())
            .collect();
        self.rules
            .iter()
            .filter(|rule| self.is_produced_by(rule, requirement_id, &resolutions))
            .collect()
    }

    pub fn for_resolution(&self, resolution_id: &StableId) -> Vec<&'a Rule> {
        self.rules
            .iter()
            .filter(|rule| {
                self.has_edge(
                    EdgeType::Produces,
                    NodeType::Resolution,
                    resolution_id,
                    NodeType::Rule,
                    &rule.id,
                )
            })
            .collect()
    }

    fn is_produced_by(
        &self,
        rule: &Rule,
        requirement_id: &StableId,
        resolutions: &BTreeSet<&str>,
    ) -> bool {
        self.edges.iter().any(|edge| {
            edge.edge_type == EdgeType::Produces
                && edge.to_type == NodeType::Rule
                && edge.to_id == rule.id
                && ((edge.from_type == NodeType::Requirement && edge.from_id == *requirement_id)
                    || (edge.from_type == NodeType::Resolution
                        && resolutions.contains(edge.from_id.as_str())))
        })
    }

    fn has_edge(
        &self,
        edge_type: EdgeType,
        from_type: NodeType,
        from_id: &StableId,
        to_type: NodeType,
        to_id: &StableId,
    ) -> bool {
        self.edges.iter().any(|edge| {
            edge.edge_type == edge_type
                && edge.from_type == from_type
                && edge.from_id == *from_id
                && edge.to_type == to_type
                && edge.to_id == *to_id
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct RulePolicy {
    pub severities: Vec<RuleSeverity>,
    pub minimum: u32,
}

impl RulePolicy {
    pub fn matches(&self, rules: &[&Rule]) -> bool {
        let selected = rules
            .iter()
            .filter(|rule| self.severities.is_empty() || self.severities.contains(&rule.severity))
            .count();
        selected >= self.minimum as usize && (self.severities.is_empty() || selected > 0)
    }
}
