use provenance_core::{
    Edge, EdgeType, NodeType, Question, QuestionStatus, Requirement, RequirementStatus, Resolution,
    ResolutionStatus, Rule, ScopeId, Source, StableId, Thread, Topic, TopicStatus,
};
use std::collections::BTreeSet;

/// Why a graph record appears on the shaping frontier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GapKind {
    /// A requirement has no domain assignment.
    MissingDomainId,
    /// A requirement has no source refs and no `references` edge.
    MissingSourceRefs,
    /// A resolved requirement has no `resolves` edge pointing at it.
    NoResolvingDecision,
    /// A resolved requirement or approved resolution produced no rule.
    NoProducedRules,
    /// No `produces` edge points at a rule.
    OrphanRule,
    /// A resolution resolves no requirement.
    OrphanResolution,
    /// A source nothing references.
    UnreferencedSource,
    /// A reference to a record that does not exist in the scope.
    DanglingReference,
    /// A `contradicts` pair has no resolving decision.
    UnresolvedContradictsPair,
    /// A question is still open.
    OpenQuestion,
    /// A topic is still unexplored.
    UnexploredTopic,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GapItem {
    pub kind: GapKind,
    pub node_type: NodeType,
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_node_type: Option<NodeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_id: Option<String>,
    pub reason: String,
}

impl GapItem {
    fn new(
        kind: GapKind,
        node_type: NodeType,
        node_id: &StableId,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            node_type,
            node_id: node_id.as_str().to_string(),
            related_node_type: None,
            related_node_id: None,
            requirement_id: (node_type == NodeType::Requirement)
                .then(|| node_id.as_str().to_string()),
            reason: reason.into(),
        }
    }

    fn with_related(mut self, node_type: NodeType, node_id: &StableId) -> Self {
        self.related_node_type = Some(node_type);
        self.related_node_id = Some(node_id.as_str().to_string());
        self
    }

    fn with_requirement(mut self, requirement_id: &StableId) -> Self {
        self.requirement_id = Some(requirement_id.as_str().to_string());
        self
    }

    pub fn subject(&self) -> String {
        let subject = format!("{} {}", node_type_word(self.node_type), self.node_id);
        match (self.related_node_type, self.related_node_id.as_deref()) {
            (Some(related_type), Some(related_id)) => {
                format!("{subject} -> {} {related_id}", node_type_word(related_type))
            }
            _ => subject,
        }
    }
}

pub const fn node_type_word(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Source => "source",
        NodeType::Requirement => "requirement",
        NodeType::Resolution => "resolution",
        NodeType::Rule => "rule",
        NodeType::Topic => "topic",
        NodeType::Question => "question",
    }
}

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

pub fn compute_gaps(graph: &GapGraph<'_>) -> Vec<GapItem> {
    let inspector = GapInspector { graph };
    let mut gaps = Vec::new();
    inspector.add_requirement_gaps(&mut gaps);
    inspector.add_resolution_gaps(&mut gaps);
    inspector.add_rule_gaps(&mut gaps);
    inspector.add_source_gaps(&mut gaps);
    inspector.add_dangling_reference_gaps(&mut gaps);
    inspector.add_contradiction_gaps(&mut gaps);
    inspector.add_question_gaps(&mut gaps);
    inspector.add_topic_gaps(&mut gaps);
    gaps
}

struct GapInspector<'a, 'graph> {
    graph: &'a GapGraph<'graph>,
}

impl GapInspector<'_, '_> {
    fn edges(&self) -> impl Iterator<Item = &Edge> {
        let scope = self.graph.scope;
        self.graph
            .edges
            .iter()
            .filter(|edge| edge.scope_id == *scope)
    }

    fn edge_exists(
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

    fn source_exists(&self, id: &StableId) -> bool {
        self.graph.sources.iter().any(|source| source.id == *id)
    }

    fn requirement_exists(&self, id: &StableId) -> bool {
        self.graph
            .requirements
            .iter()
            .any(|requirement| requirement.id == *id)
    }

    fn resolution_exists(&self, id: &StableId) -> bool {
        self.graph
            .resolutions
            .iter()
            .any(|resolution| resolution.id == *id)
    }

    fn rule_exists(&self, id: &StableId) -> bool {
        self.graph.rules.iter().any(|rule| rule.id == *id)
    }

    fn topic_exists(&self, id: &StableId) -> bool {
        self.graph.topics.iter().any(|topic| topic.id == *id)
    }

    fn question_exists(&self, id: &StableId) -> bool {
        self.graph
            .questions
            .iter()
            .any(|question| question.id == *id)
    }

    fn node_exists(&self, node_type: NodeType, id: &StableId) -> bool {
        match node_type {
            NodeType::Source => self.source_exists(id),
            NodeType::Requirement => self.requirement_exists(id),
            NodeType::Resolution => self.resolution_exists(id),
            NodeType::Rule => self.rule_exists(id),
            NodeType::Topic => self.topic_exists(id),
            NodeType::Question => self.question_exists(id),
        }
    }

    fn resolving_resolutions(&self, requirement_id: &StableId) -> Vec<&Resolution> {
        self.graph
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

    fn resolution_resolves_any_requirement(&self, resolution_id: &StableId) -> bool {
        self.graph.requirements.iter().any(|requirement| {
            self.edge_exists(
                EdgeType::Resolves,
                NodeType::Resolution,
                resolution_id,
                NodeType::Requirement,
                &requirement.id,
            )
        })
    }

    fn produced_rules_for_requirement(&self, requirement_id: &StableId) -> Vec<&Rule> {
        let resolution_ids: BTreeSet<&str> = self
            .resolving_resolutions(requirement_id)
            .into_iter()
            .map(|resolution| resolution.id.as_str())
            .collect();
        self.graph
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

    fn produced_rules_for_resolution(&self, resolution_id: &StableId) -> Vec<&Rule> {
        self.graph
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

    fn rule_has_existing_producer(&self, rule_id: &StableId) -> bool {
        self.graph.requirements.iter().any(|requirement| {
            self.edge_exists(
                EdgeType::Produces,
                NodeType::Requirement,
                &requirement.id,
                NodeType::Rule,
                rule_id,
            )
        }) || self.graph.resolutions.iter().any(|resolution| {
            self.edge_exists(
                EdgeType::Produces,
                NodeType::Resolution,
                &resolution.id,
                NodeType::Rule,
                rule_id,
            )
        })
    }

    fn requirement_has_valid_source(&self, requirement: &Requirement) -> bool {
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

    fn source_is_referenced(&self, source_id: &StableId) -> bool {
        self.graph.requirements.iter().any(|requirement| {
            requirement
                .source_refs
                .iter()
                .any(|reference| reference.source_id == *source_id)
        }) || self.graph.requirements.iter().any(|requirement| {
            self.edge_exists(
                EdgeType::References,
                NodeType::Source,
                source_id,
                NodeType::Requirement,
                &requirement.id,
            )
        })
    }

    fn add_requirement_gaps(&self, gaps: &mut Vec<GapItem>) {
        for requirement in self.graph.requirements {
            let resolving = self.resolving_resolutions(&requirement.id);
            let resolved = requirement.status == RequirementStatus::Resolved;
            if requirement.domain_id.is_none() {
                gaps.push(GapItem::new(
                    GapKind::MissingDomainId,
                    NodeType::Requirement,
                    &requirement.id,
                    "requirement has no domain_id",
                ));
            }
            if !self.requirement_has_valid_source(requirement) {
                gaps.push(GapItem::new(
                    GapKind::MissingSourceRefs,
                    NodeType::Requirement,
                    &requirement.id,
                    "requirement has no source refs",
                ));
            }
            if resolved && resolving.is_empty() {
                gaps.push(GapItem::new(
                    GapKind::NoResolvingDecision,
                    NodeType::Requirement,
                    &requirement.id,
                    "resolved requirement has no resolving decision",
                ));
            }
            if (resolved || !resolving.is_empty())
                && self
                    .produced_rules_for_requirement(&requirement.id)
                    .is_empty()
            {
                gaps.push(GapItem::new(
                    GapKind::NoProducedRules,
                    NodeType::Requirement,
                    &requirement.id,
                    "resolved requirement has no downstream rule",
                ));
            }
        }
    }

    fn add_resolution_gaps(&self, gaps: &mut Vec<GapItem>) {
        for resolution in self.graph.resolutions {
            if !self.resolution_resolves_any_requirement(&resolution.id) {
                gaps.push(GapItem::new(
                    GapKind::OrphanResolution,
                    NodeType::Resolution,
                    &resolution.id,
                    "resolution does not resolve any requirement",
                ));
            }
            if resolution.status == ResolutionStatus::Approved
                && self
                    .produced_rules_for_resolution(&resolution.id)
                    .is_empty()
            {
                gaps.push(GapItem::new(
                    GapKind::NoProducedRules,
                    NodeType::Resolution,
                    &resolution.id,
                    "approved resolution produced no rules",
                ));
            }
        }
    }

    fn add_rule_gaps(&self, gaps: &mut Vec<GapItem>) {
        for rule in self.graph.rules {
            if !self.rule_has_existing_producer(&rule.id) {
                gaps.push(GapItem::new(
                    GapKind::OrphanRule,
                    NodeType::Rule,
                    &rule.id,
                    "no resolution or requirement produces this rule",
                ));
            }
        }
    }

    fn add_source_gaps(&self, gaps: &mut Vec<GapItem>) {
        for source in self.graph.sources {
            if !self.source_is_referenced(&source.id) {
                gaps.push(GapItem::new(
                    GapKind::UnreferencedSource,
                    NodeType::Source,
                    &source.id,
                    "no requirement references this source",
                ));
            }
        }
    }

    fn add_dangling_reference_gaps(&self, gaps: &mut Vec<GapItem>) {
        self.add_dangling_requirement_source_refs(gaps);
        self.add_dangling_source_refs(gaps);
        self.add_dangling_resolution_refs(gaps);
        self.add_dangling_topic_refs(gaps);
        self.add_dangling_question_refs(gaps);
        self.add_dangling_thread_refs(gaps);
        self.add_dangling_edge_refs(gaps);
    }

    fn add_dangling_requirement_source_refs(&self, gaps: &mut Vec<GapItem>) {
        for requirement in self.graph.requirements {
            for reference in &requirement.source_refs {
                if !self.source_exists(&reference.source_id) {
                    gaps.push(
                        GapItem::new(
                            GapKind::DanglingReference,
                            NodeType::Requirement,
                            &requirement.id,
                            format!(
                                "source ref points at missing source {}",
                                reference.source_id.as_str()
                            ),
                        )
                        .with_related(NodeType::Source, &reference.source_id),
                    );
                }
            }
        }
    }

    fn add_dangling_source_refs(&self, gaps: &mut Vec<GapItem>) {
        for source in self.graph.sources {
            if let Some(id) = &source.superseded_by {
                if !self.source_exists(id) {
                    gaps.push(
                        GapItem::new(
                            GapKind::DanglingReference,
                            NodeType::Source,
                            &source.id,
                            format!("superseded by missing source {}", id.as_str()),
                        )
                        .with_related(NodeType::Source, id),
                    );
                }
            }
        }
    }

    fn add_dangling_resolution_refs(&self, gaps: &mut Vec<GapItem>) {
        for resolution in self.graph.resolutions {
            if let Some(id) = &resolution.superseded_by {
                if !self.resolution_exists(id) {
                    gaps.push(
                        GapItem::new(
                            GapKind::DanglingReference,
                            NodeType::Resolution,
                            &resolution.id,
                            format!("superseded by missing resolution {}", id.as_str()),
                        )
                        .with_related(NodeType::Resolution, id),
                    );
                }
            }
        }
    }

    fn add_dangling_topic_refs(&self, gaps: &mut Vec<GapItem>) {
        for topic in self.graph.topics {
            if !self.requirement_exists(&topic.requirement_id) {
                gaps.push(
                    GapItem::new(
                        GapKind::DanglingReference,
                        NodeType::Topic,
                        &topic.id,
                        format!(
                            "topic points at missing requirement {}",
                            topic.requirement_id.as_str()
                        ),
                    )
                    .with_related(NodeType::Requirement, &topic.requirement_id),
                );
            }
        }
    }

    fn add_dangling_question_refs(&self, gaps: &mut Vec<GapItem>) {
        for question in self.graph.questions {
            if !self.topic_exists(&question.topic_id) {
                gaps.push(
                    GapItem::new(
                        GapKind::DanglingReference,
                        NodeType::Question,
                        &question.id,
                        format!(
                            "question points at missing topic {}",
                            question.topic_id.as_str()
                        ),
                    )
                    .with_related(NodeType::Topic, &question.topic_id)
                    .with_requirement(&question.requirement_id),
                );
            }
            if !self.requirement_exists(&question.requirement_id) {
                gaps.push(
                    GapItem::new(
                        GapKind::DanglingReference,
                        NodeType::Question,
                        &question.id,
                        format!(
                            "question points at missing requirement {}",
                            question.requirement_id.as_str()
                        ),
                    )
                    .with_related(NodeType::Requirement, &question.requirement_id),
                );
            }
            if let Some(id) = &question.resolution_id {
                if !self.resolution_exists(id) {
                    gaps.push(
                        GapItem::new(
                            GapKind::DanglingReference,
                            NodeType::Question,
                            &question.id,
                            format!("question points at missing resolution {}", id.as_str()),
                        )
                        .with_related(NodeType::Resolution, id)
                        .with_requirement(&question.requirement_id),
                    );
                }
            }
        }
    }

    fn add_dangling_thread_refs(&self, gaps: &mut Vec<GapItem>) {
        for thread in self.graph.threads {
            if !self.node_exists(thread.parent.node_type, &thread.parent.node_id) {
                gaps.push(GapItem::new(
                    GapKind::DanglingReference,
                    thread.parent.node_type,
                    &thread.parent.node_id,
                    format!(
                        "thread {} points at missing {} {}",
                        thread.id.as_str(),
                        node_type_word(thread.parent.node_type),
                        thread.parent.node_id.as_str()
                    ),
                ));
            }
        }
    }

    fn add_dangling_edge_refs(&self, gaps: &mut Vec<GapItem>) {
        for edge in self.edges() {
            if !self.node_exists(edge.from_type, &edge.from_id) {
                gaps.push(GapItem::new(
                    GapKind::DanglingReference,
                    edge.from_type,
                    &edge.from_id,
                    format!(
                        "edge {} points from missing {} {}",
                        edge.id.as_str(),
                        node_type_word(edge.from_type),
                        edge.from_id.as_str()
                    ),
                ));
            }
            if !self.node_exists(edge.to_type, &edge.to_id) {
                gaps.push(
                    GapItem::new(
                        GapKind::DanglingReference,
                        edge.to_type,
                        &edge.to_id,
                        format!(
                            "edge {} points to missing {} {}",
                            edge.id.as_str(),
                            node_type_word(edge.to_type),
                            edge.to_id.as_str()
                        ),
                    )
                    .with_related(edge.from_type, &edge.from_id),
                );
            }
        }
    }

    fn add_contradiction_gaps(&self, gaps: &mut Vec<GapItem>) {
        let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
        for edge in self.edges().filter(|edge| {
            edge.edge_type == EdgeType::Contradicts
                && edge.from_type == NodeType::Requirement
                && edge.to_type == NodeType::Requirement
                && self.requirement_exists(&edge.from_id)
                && self.requirement_exists(&edge.to_id)
        }) {
            let pair = ordered_pair(&edge.from_id, &edge.to_id);
            if !seen.insert(pair) || self.contradiction_is_resolved(&edge.from_id, &edge.to_id) {
                continue;
            }
            gaps.push(
                GapItem::new(
                    GapKind::UnresolvedContradictsPair,
                    NodeType::Requirement,
                    &edge.from_id,
                    "unresolved `contradicts` pair",
                )
                .with_related(NodeType::Requirement, &edge.to_id),
            );
        }
    }

    fn contradiction_is_resolved(&self, left_id: &StableId, right_id: &StableId) -> bool {
        if self.edge_exists(
            EdgeType::Supersedes,
            NodeType::Requirement,
            left_id,
            NodeType::Requirement,
            right_id,
        ) || self.edge_exists(
            EdgeType::Supersedes,
            NodeType::Requirement,
            right_id,
            NodeType::Requirement,
            left_id,
        ) {
            return true;
        }
        let left_resolutions: BTreeSet<&str> = self
            .resolving_resolutions(left_id)
            .into_iter()
            .map(|resolution| resolution.id.as_str())
            .collect();
        self.resolving_resolutions(right_id)
            .into_iter()
            .any(|resolution| left_resolutions.contains(resolution.id.as_str()))
    }

    fn add_question_gaps(&self, gaps: &mut Vec<GapItem>) {
        for question in self.graph.questions.iter().filter(|question| {
            matches!(
                question.status,
                QuestionStatus::Open | QuestionStatus::BlockedOnHuman
            )
        }) {
            let reason = match question.status {
                QuestionStatus::Open => "open question",
                QuestionStatus::BlockedOnHuman => "blocked_on_human question",
                QuestionStatus::Answered => unreachable!("answered questions are filtered out"),
            };
            gaps.push(
                GapItem::new(
                    GapKind::OpenQuestion,
                    NodeType::Question,
                    &question.id,
                    reason,
                )
                .with_requirement(&question.requirement_id),
            );
        }
    }

    fn add_topic_gaps(&self, gaps: &mut Vec<GapItem>) {
        for topic in self
            .graph
            .topics
            .iter()
            .filter(|topic| topic.status == TopicStatus::Open)
        {
            gaps.push(
                GapItem::new(
                    GapKind::UnexploredTopic,
                    NodeType::Topic,
                    &topic.id,
                    "unexplored topic",
                )
                .with_requirement(&topic.requirement_id),
            );
        }
    }
}

fn ordered_pair<'a>(left: &'a StableId, right: &'a StableId) -> (&'a str, &'a str) {
    if left.as_str() <= right.as_str() {
        (left.as_str(), right.as_str())
    } else {
        (right.as_str(), left.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use provenance_core::{
        Edge, ResolutionMethod, RuleSeverity, RuleStatus, SchemaVersion, SourceReference,
        SourceType,
    };

    fn sid(value: &str) -> StableId {
        StableId::new(value).unwrap()
    }

    fn scope_id() -> ScopeId {
        ScopeId::new("default").unwrap()
    }

    fn requirement(id: &str) -> Requirement {
        Requirement {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            statement: format!("{id} statement"),
            description: None,
            fog: None,
            status: RequirementStatus::Active,
            domain_id: None,
            source_refs: Vec::new(),
            origin_thread: None,
            origin_message: None,
        }
    }

    fn resolution(id: &str) -> Resolution {
        Resolution {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            title: id.to_string(),
            position: "Adopt the decision".to_string(),
            rationale: "It resolves the pair".to_string(),
            status: ResolutionStatus::Draft,
            context: None,
            enforcement: None,
            confidence: None,
            inputs: Vec::new(),
            made_by: None,
            approved_by: None,
            approved_at: None,
            superseded_by: None,
            review_on: None,
            review_triggers: serde_json::json!([]),
            origin_thread: None,
            origin_message: None,
        }
    }

    fn rule(id: &str) -> Rule {
        Rule {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            rule_code: id.to_string(),
            name: None,
            description: None,
            statement: "Rule statement".to_string(),
            status: RuleStatus::Active,
            severity: RuleSeverity::High,
            rule_type: None,
            modality: None,
            confidence: None,
            extraction_method: None,
            source_document: None,
            source_section: None,
            origin_thread: None,
            origin_message: None,
            expression: serde_json::json!({}),
            inputs: serde_json::json!([]),
        }
    }

    fn topic(id: &str, status: TopicStatus) -> Topic {
        Topic {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            requirement_id: sid("req_topic"),
            title: id.to_string(),
            status,
            claimed_by: None,
            claimed_at: None,
            links: Vec::new(),
        }
    }

    fn question(id: &str, topic_id: &str, status: QuestionStatus) -> Question {
        Question {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            topic_id: sid(topic_id),
            requirement_id: sid("req_topic"),
            question: "What remains?".to_string(),
            resolution_method: ResolutionMethod::Grill,
            status,
            claimed_by: None,
            claimed_at: None,
            answer: (status == QuestionStatus::Answered).then(|| "Done".to_string()),
            links: Vec::new(),
            resolution_id: None,
        }
    }

    fn source(id: &str) -> Source {
        Source {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            name: id.to_string(),
            source_type: SourceType::Policy,
            url: None,
            reference: None,
            commit_pin: None,
            effective_date: None,
            review_date: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        }
    }

    fn edge(edge_type: EdgeType, from: (NodeType, &str), to: (NodeType, &str)) -> Edge {
        Edge {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: Edge::stable_id(edge_type, from.0, &sid(from.1), to.0, &sid(to.1)).unwrap(),
            edge_type,
            from_type: from.0,
            from_id: sid(from.1),
            to_type: to.0,
            to_id: sid(to.1),
            label: None,
        }
    }

    fn compute_for(
        sources: &[Source],
        requirements: &[Requirement],
        resolutions: &[Resolution],
        rules: &[Rule],
        topics: &[Topic],
        questions: &[Question],
        edges: &[Edge],
    ) -> Vec<GapItem> {
        let scope = scope_id();
        let threads = Vec::new();
        compute_gaps(&GapGraph {
            scope: &scope,
            sources,
            requirements,
            resolutions,
            rules,
            topics,
            questions,
            edges,
            threads: &threads,
        })
    }

    fn count_kind(gaps: &[GapItem], kind: GapKind) -> usize {
        gaps.iter().filter(|gap| gap.kind == kind).count()
    }

    #[test]
    fn shared_resolving_resolution_suppresses_unresolved_contradiction_gap() {
        let requirements = vec![requirement("req_left"), requirement("req_right")];
        let resolutions = vec![resolution("res_shared")];
        let edges = vec![
            edge(
                EdgeType::Contradicts,
                (NodeType::Requirement, "req_left"),
                (NodeType::Requirement, "req_right"),
            ),
            edge(
                EdgeType::Resolves,
                (NodeType::Resolution, "res_shared"),
                (NodeType::Requirement, "req_left"),
            ),
            edge(
                EdgeType::Resolves,
                (NodeType::Resolution, "res_shared"),
                (NodeType::Requirement, "req_right"),
            ),
        ];

        let gaps = compute_for(&[], &requirements, &resolutions, &[], &[], &[], &edges);

        assert_eq!(count_kind(&gaps, GapKind::UnresolvedContradictsPair), 0);
    }

    #[test]
    fn supersedes_edge_suppresses_unresolved_contradiction_gap() {
        let requirements = vec![requirement("req_left"), requirement("req_right")];
        let edges = vec![
            edge(
                EdgeType::Contradicts,
                (NodeType::Requirement, "req_left"),
                (NodeType::Requirement, "req_right"),
            ),
            edge(
                EdgeType::Supersedes,
                (NodeType::Requirement, "req_right"),
                (NodeType::Requirement, "req_left"),
            ),
        ];

        let gaps = compute_for(&[], &requirements, &[], &[], &[], &[], &edges);

        assert_eq!(count_kind(&gaps, GapKind::UnresolvedContradictsPair), 0);
    }

    #[test]
    fn answered_questions_and_explored_topics_are_not_frontier_gaps() {
        let requirements = vec![requirement("req_topic")];
        let topics = vec![topic("topic_explored", TopicStatus::Explored)];
        let questions = vec![question(
            "question_answered",
            "topic_explored",
            QuestionStatus::Answered,
        )];

        let gaps = compute_for(&[], &requirements, &[], &[], &topics, &questions, &[]);

        assert_eq!(count_kind(&gaps, GapKind::OpenQuestion), 0);
        assert_eq!(count_kind(&gaps, GapKind::UnexploredTopic), 0);
    }

    #[test]
    fn rule_produced_by_missing_resolution_is_orphaned_and_has_dangling_edge_gap() {
        let rules = vec![rule("rule_orphaned")];
        let edges = vec![edge(
            EdgeType::Produces,
            (NodeType::Resolution, "res_missing"),
            (NodeType::Rule, "rule_orphaned"),
        )];

        let gaps = compute_for(&[], &[], &[], &rules, &[], &[], &edges);

        assert!(gaps.iter().any(|gap| {
            gap.kind == GapKind::OrphanRule
                && gap.node_type == NodeType::Rule
                && gap.node_id == "rule_orphaned"
        }));
        assert!(gaps.iter().any(|gap| {
            gap.kind == GapKind::DanglingReference
                && gap.node_type == NodeType::Resolution
                && gap.node_id == "res_missing"
                && gap.reason.contains("edge")
        }));
    }

    #[test]
    fn unresolved_contradiction_pair_is_reported_once_for_bidirectional_edges() {
        let sources = vec![source("source_anchor")];
        let mut requirements = vec![requirement("req_left"), requirement("req_right")];
        for requirement in &mut requirements {
            requirement.source_refs = vec![SourceReference {
                source_id: sid("source_anchor"),
                clause: None,
            }];
        }
        let edges = vec![
            edge(
                EdgeType::Contradicts,
                (NodeType::Requirement, "req_left"),
                (NodeType::Requirement, "req_right"),
            ),
            edge(
                EdgeType::Contradicts,
                (NodeType::Requirement, "req_right"),
                (NodeType::Requirement, "req_left"),
            ),
        ];

        let gaps = compute_for(&sources, &requirements, &[], &[], &[], &[], &edges);

        assert_eq!(count_kind(&gaps, GapKind::UnresolvedContradictsPair), 1);
    }
}
