use provenance_core::{
    Edge, EdgeType, NodeType, Requirement, Resolution, Rule, RuleSeverity, ScopeId, StableId,
};
use std::collections::BTreeSet;

/// Canonical graph query for rules produced downstream of requirements/resolutions.
pub struct DownstreamRuleQuery<'a> {
    scope: &'a ScopeId,
    edges: &'a [Edge],
    requirements: &'a [Requirement],
    resolutions: &'a [Resolution],
    rules: &'a [Rule],
}

impl<'a> DownstreamRuleQuery<'a> {
    pub const fn new(
        scope: &'a ScopeId,
        edges: &'a [Edge],
        requirements: &'a [Requirement],
        resolutions: &'a [Resolution],
        rules: &'a [Rule],
    ) -> Self {
        Self {
            scope,
            edges,
            requirements,
            resolutions,
            rules,
        }
    }

    pub fn for_requirement(&self, requirement_id: &StableId) -> Vec<&'a Rule> {
        if !self.requirements.iter().any(|requirement| {
            requirement.scope_id == *self.scope && requirement.id == *requirement_id
        }) {
            return Vec::new();
        }
        let resolutions: BTreeSet<_> = self
            .resolutions
            .iter()
            .filter(|resolution| resolution.scope_id == *self.scope)
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
            .filter(|rule| rule.scope_id == *self.scope)
            .filter(|rule| self.is_produced_by(rule, requirement_id, &resolutions))
            .collect()
    }

    pub fn for_resolution(&self, resolution_id: &StableId) -> Vec<&'a Rule> {
        if !self
            .resolutions
            .iter()
            .any(|resolution| resolution.scope_id == *self.scope && resolution.id == *resolution_id)
        {
            return Vec::new();
        }
        self.rules
            .iter()
            .filter(|rule| rule.scope_id == *self.scope)
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
            edge.scope_id == *self.scope
                && edge.edge_type == EdgeType::Produces
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
            edge.scope_id == *self.scope
                && edge.edge_type == edge_type
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

#[cfg(test)]
mod tests {
    use super::DownstreamRuleQuery;
    use provenance_core::{Edge, Requirement, Rule, ScopeId, StableId};

    fn requirement(scope: &str) -> Requirement {
        serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "scope_id": scope,
            "id": "req_current",
            "statement": "requirement",
            "status": "active"
        }))
        .unwrap()
    }

    fn rule(scope: &str, id: &str) -> Rule {
        serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "scope_id": scope,
            "id": id,
            "rule_code": id,
            "statement": "rule",
            "status": "active",
            "severity": "high"
        }))
        .unwrap()
    }

    fn edge(scope: &str, rule_id: &str) -> Edge {
        serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "scope_id": scope,
            "id": format!("edge_{scope}_{rule_id}"),
            "edge_type": "produces",
            "from_type": "requirement",
            "from_id": "req_current",
            "to_type": "rule",
            "to_id": rule_id
        }))
        .unwrap()
    }

    #[test]
    fn foreign_scope_rule_is_not_downstream_of_current_requirement() {
        let scope = ScopeId::new("current").unwrap();
        let requirements = [requirement("current")];
        let rules = [rule("foreign", "rule_foreign")];
        let edges = [edge("current", "rule_foreign")];
        let query = DownstreamRuleQuery::new(&scope, &edges, &requirements, &[], &rules);

        assert!(query
            .for_requirement(&StableId::new("req_current").unwrap())
            .is_empty());
    }

    #[test]
    fn foreign_scope_edge_does_not_connect_current_scope_rule() {
        let scope = ScopeId::new("current").unwrap();
        let requirements = [requirement("current")];
        let rules = [rule("current", "rule_current")];
        let edges = [edge("foreign", "rule_current")];
        let query = DownstreamRuleQuery::new(&scope, &edges, &requirements, &[], &rules);

        assert!(query
            .for_requirement(&StableId::new("req_current").unwrap())
            .is_empty());
    }
}
