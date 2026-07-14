use crate::cache::DownstreamRuleQuery;
use provenance_core::{
    Edge, EdgeType, NodeType, Question, Requirement, Resolution, Rule, ScopeId, Source, StableId,
    Thread, Topic,
};

pub struct GapGraph<'a> {
    pub scope: &'a ScopeId,
    pub sources: &'a [Source],
    pub requirements: &'a [Requirement],
    pub resolutions: &'a [Resolution],
    pub rules: &'a [Rule],
    pub topics: &'a [Topic],
    pub questions: &'a [Question],
    pub edges: &'a [Edge],
    pub threads: &'a [Thread],
}

pub(super) struct GraphQuery<'a, 'graph> {
    graph: &'a GapGraph<'graph>,
}

impl<'a, 'graph> GraphQuery<'a, 'graph> {
    pub const fn new(graph: &'a GapGraph<'graph>) -> Self {
        Self { graph }
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        let scope = self.graph.scope;
        self.graph
            .edges
            .iter()
            .filter(move |edge| edge.scope_id == *scope)
    }

    pub fn sources(&self) -> impl Iterator<Item = &Source> {
        let scope = self.graph.scope;
        self.graph
            .sources
            .iter()
            .filter(move |source| source.scope_id == *scope)
    }

    pub fn requirements(&self) -> impl Iterator<Item = &Requirement> {
        let scope = self.graph.scope;
        self.graph
            .requirements
            .iter()
            .filter(move |requirement| requirement.scope_id == *scope)
    }

    pub fn resolutions(&self) -> impl Iterator<Item = &Resolution> {
        let scope = self.graph.scope;
        self.graph
            .resolutions
            .iter()
            .filter(move |resolution| resolution.scope_id == *scope)
    }

    pub fn rules(&self) -> impl Iterator<Item = &Rule> {
        let scope = self.graph.scope;
        self.graph
            .rules
            .iter()
            .filter(move |rule| rule.scope_id == *scope)
    }

    pub fn topics(&self) -> impl Iterator<Item = &Topic> {
        let scope = self.graph.scope;
        self.graph
            .topics
            .iter()
            .filter(move |topic| topic.scope_id == *scope)
    }

    pub fn questions(&self) -> impl Iterator<Item = &Question> {
        let scope = self.graph.scope;
        self.graph
            .questions
            .iter()
            .filter(move |question| question.scope_id == *scope)
    }

    pub fn threads(&self) -> impl Iterator<Item = &Thread> {
        let scope = self.graph.scope;
        self.graph
            .threads
            .iter()
            .filter(move |thread| thread.scope_id == *scope)
    }

    pub fn edge_exists(
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

    pub fn source_exists(&self, id: &StableId) -> bool {
        self.sources().any(|source| source.id == *id)
    }

    pub fn requirement_exists(&self, id: &StableId) -> bool {
        self.requirements().any(|requirement| requirement.id == *id)
    }

    pub fn resolution_exists(&self, id: &StableId) -> bool {
        self.resolutions().any(|resolution| resolution.id == *id)
    }

    pub fn topic_exists(&self, id: &StableId) -> bool {
        self.topics().any(|topic| topic.id == *id)
    }

    pub fn node_exists(&self, node_type: NodeType, id: &StableId) -> bool {
        match node_type {
            NodeType::Source => self.source_exists(id),
            NodeType::Requirement => self.requirement_exists(id),
            NodeType::Resolution => self.resolution_exists(id),
            NodeType::Rule => self.rules().any(|rule| rule.id == *id),
            NodeType::Topic => self.topic_exists(id),
            NodeType::Question => self.questions().any(|question| question.id == *id),
        }
    }

    pub fn resolving_resolutions(&self, requirement_id: &StableId) -> Vec<&Resolution> {
        self.resolutions()
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

    pub fn resolution_resolves_any_requirement(&self, resolution_id: &StableId) -> bool {
        self.requirements().any(|requirement| {
            self.edge_exists(
                EdgeType::Resolves,
                NodeType::Resolution,
                resolution_id,
                NodeType::Requirement,
                &requirement.id,
            )
        })
    }

    pub fn produced_rules_for_requirement(&self, requirement_id: &StableId) -> Vec<&Rule> {
        DownstreamRuleQuery::new(
            self.graph.scope,
            self.graph.edges,
            self.graph.requirements,
            self.graph.resolutions,
            self.graph.rules,
        )
        .for_requirement(requirement_id)
    }

    pub fn produced_rules_for_resolution(&self, resolution_id: &StableId) -> Vec<&Rule> {
        DownstreamRuleQuery::new(
            self.graph.scope,
            self.graph.edges,
            self.graph.requirements,
            self.graph.resolutions,
            self.graph.rules,
        )
        .for_resolution(resolution_id)
    }

    pub fn rule_has_existing_producer(&self, rule_id: &StableId) -> bool {
        self.requirements().any(|requirement| {
            self.edge_exists(
                EdgeType::Produces,
                NodeType::Requirement,
                &requirement.id,
                NodeType::Rule,
                rule_id,
            )
        }) || self.resolutions().any(|resolution| {
            self.edge_exists(
                EdgeType::Produces,
                NodeType::Resolution,
                &resolution.id,
                NodeType::Rule,
                rule_id,
            )
        })
    }

    pub fn requirement_has_valid_source(&self, requirement: &Requirement) -> bool {
        requirement
            .source_refs
            .iter()
            .any(|reference| self.source_exists(&reference.source_id))
            || self.edges().any(|edge| {
                edge.edge_type == EdgeType::References
                    && edge.from_type == NodeType::Source
                    && self.source_exists(&edge.from_id)
                    && edge.to_type == NodeType::Requirement
                    && edge.to_id == requirement.id
            })
    }

    pub fn source_is_referenced(&self, source_id: &StableId) -> bool {
        self.requirements().any(|requirement| {
            requirement
                .source_refs
                .iter()
                .any(|reference| reference.source_id == *source_id)
        }) || self.requirements().any(|requirement| {
            self.edge_exists(
                EdgeType::References,
                NodeType::Source,
                source_id,
                NodeType::Requirement,
                &requirement.id,
            )
        })
    }
}
