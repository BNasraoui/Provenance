use crate::cache::{compute_gaps, GapGraph, GapItem};
use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{Edge, EdgeType, NodeType, Requirement, Resolution, Rule, Source};

#[derive(Debug, serde::Serialize)]
pub struct TraceabilityView {
    pub rule: Rule,
    pub resolutions: Vec<Resolution>,
    pub requirements: Vec<Requirement>,
    pub sources: Vec<Source>,
    pub edges: Vec<Edge>,
}

pub fn trace_rule(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    rule_id: &provenance_core::StableId,
) -> anyhow::Result<TraceabilityView> {
    let store = StateStore::new(layout.clone());
    let rule = store
        .list_rules(scope)?
        .into_iter()
        .find(|rule| rule.id == *rule_id)
        .ok_or_else(|| anyhow::anyhow!("rule not found"))?;
    let edges: Vec<Edge> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| edge.scope_id == *scope)
        .collect();
    let resolution_ids: Vec<_> = edges
        .iter()
        .filter(|edge| {
            edge.edge_type == EdgeType::Produces
                && edge.from_type == NodeType::Resolution
                && edge.to_type == NodeType::Rule
                && edge.to_id == *rule_id
        })
        .map(|edge| edge.from_id.clone())
        .collect();
    let requirement_ids: Vec<_> = edges
        .iter()
        .filter(|edge| {
            (edge.edge_type == EdgeType::Produces
                && edge.from_type == NodeType::Requirement
                && edge.to_type == NodeType::Rule
                && edge.to_id == *rule_id)
                || (edge.edge_type == EdgeType::Resolves
                    && edge.from_type == NodeType::Resolution
                    && resolution_ids.iter().any(|id| id == &edge.from_id))
        })
        .map(|edge| edge.to_id.clone())
        .collect();
    let source_ids: Vec<_> = edges
        .iter()
        .filter(|edge| {
            edge.edge_type == EdgeType::References
                && edge.from_type == NodeType::Source
                && requirement_ids.iter().any(|id| id == &edge.to_id)
        })
        .map(|edge| edge.from_id.clone())
        .collect();
    let resolutions = store
        .list_resolutions(scope)?
        .into_iter()
        .filter(|resolution| resolution_ids.iter().any(|id| id == &resolution.id))
        .collect();
    let requirements = store
        .list_requirements(scope)?
        .into_iter()
        .filter(|requirement| requirement_ids.iter().any(|id| id == &requirement.id))
        .collect();
    let sources = store
        .list_sources(scope)?
        .into_iter()
        .filter(|source| source_ids.iter().any(|id| id == &source.id))
        .collect();
    Ok(TraceabilityView {
        rule,
        resolutions,
        requirements,
        sources,
        edges,
    })
}

pub fn find_gaps(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
) -> anyhow::Result<Vec<GapItem>> {
    let store = StateStore::new(layout.clone());
    let edges = store.list_edges()?;
    let sources = store.list_sources(scope)?;
    let requirements = store.list_requirements(scope)?;
    let resolutions = store.list_resolutions(scope)?;
    let rules = store.list_rules(scope)?;
    let topics = store.list_topics(scope)?;
    let questions = store.list_questions(scope)?;
    let threads = store.list_threads(scope)?;
    Ok(compute_gaps(&GapGraph {
        scope,
        sources: &sources,
        requirements: &requirements,
        resolutions: &resolutions,
        rules: &rules,
        topics: &topics,
        questions: &questions,
        edges: &edges,
        threads: &threads,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::{prime_context, render_prime_markdown, GapKind},
        state_store::{
            AddSourceReferenceInput, CreateEdgeInput, CreateQuestionInput, CreateRequirementInput,
            CreateResolutionInput, CreateRuleInput, CreateSourceInput, CreateTopicInput,
        },
    };
    use provenance_core::{
        EdgeType, Manifest, NodeType, QuestionStatus, RepoPathPrefix, RequirementStatus,
        ResolutionMethod, ResolutionStatus, RuleSeverity, RuleStatus, ScopeId, SourceType,
        StableId, TopicStatus,
    };

    fn sid(value: &str) -> StableId {
        StableId::new(value).unwrap()
    }

    fn seeded_layout() -> (tempfile::TempDir, ProvenanceLayout, ScopeId) {
        let dir = tempfile::tempdir().unwrap();
        let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
        let scope = ScopeId::new("default").unwrap();
        std::fs::write(
            layout.manifest_path(),
            serde_json::to_string(&Manifest::default_with_scope(
                scope.clone(),
                RepoPathPrefix::new("."),
            ))
            .unwrap(),
        )
        .unwrap();
        (dir, layout, scope)
    }

    fn create_source(store: &StateStore, scope: &ScopeId, id: &str) {
        store
            .create_source(CreateSourceInput {
                scope_id: scope.clone(),
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
            })
            .unwrap();
    }

    fn create_requirement(
        store: &StateStore,
        scope: &ScopeId,
        id: &str,
        status: RequirementStatus,
    ) {
        store
            .create_requirement(CreateRequirementInput {
                scope_id: scope.clone(),
                id: sid(id),
                statement: format!("{id} statement"),
                description: None,
                status,
                domain_id: None,
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
    }

    fn attach_source(store: &StateStore, scope: &ScopeId, requirement_id: &str) {
        store
            .add_source_reference(AddSourceReferenceInput {
                scope_id: scope.clone(),
                source_id: sid("source_anchor"),
                requirement_id: sid(requirement_id),
                clause: None,
            })
            .unwrap();
    }

    fn create_resolution(
        store: &StateStore,
        scope: &ScopeId,
        id: &str,
        requirement_id: Option<&str>,
    ) {
        store
            .create_resolution(CreateResolutionInput {
                scope_id: scope.clone(),
                id: sid(id),
                title: id.to_string(),
                requirement_id: requirement_id.map(sid),
                position: "Adopt the decision".to_string(),
                rationale: "It resolves the frontier".to_string(),
                status: ResolutionStatus::Approved,
                context: None,
                enforcement: None,
                confidence: None,
                inputs: Vec::new(),
                made_by: None,
                approved_by: None,
                approved_at: None,
                superseded_by: None,
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
    }

    fn assert_gap(gaps: &[GapItem], kind: GapKind, node_type: NodeType, node_id: &str) {
        assert!(
            gaps.iter().any(|gap| {
                gap.kind == kind && gap.node_type == node_type && gap.node_id == node_id
            }),
            "missing {kind:?} for {node_type:?} {node_id}; got {gaps:#?}"
        );
    }

    #[test]
    fn find_gaps_reports_the_frontier_taxonomy() {
        let (_dir, layout, scope) = seeded_layout();
        let store = StateStore::new(layout.clone());
        create_source(&store, &scope, "source_anchor");
        create_source(&store, &scope, "source_unused");
        store
            .create_source(CreateSourceInput {
                scope_id: scope.clone(),
                id: sid("source_dangling"),
                name: "source_dangling".to_string(),
                source_type: SourceType::Policy,
                url: None,
                reference: None,
                commit_pin: None,
                effective_date: None,
                review_date: None,
                superseded_by: Some(sid("source_missing")),
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();

        for (id, status) in [
            ("req_missing_source", RequirementStatus::Active),
            ("req_resolved_no_decision", RequirementStatus::Resolved),
            ("req_decided_no_rule", RequirementStatus::Active),
            ("req_contradicts_a", RequirementStatus::Active),
            ("req_contradicts_b", RequirementStatus::Active),
            ("req_question_topic", RequirementStatus::Active),
        ] {
            create_requirement(&store, &scope, id, status);
            if id != "req_missing_source" {
                attach_source(&store, &scope, id);
            }
        }
        store
            .add_source_reference(AddSourceReferenceInput {
                scope_id: scope.clone(),
                source_id: sid("source_dangling"),
                requirement_id: sid("req_question_topic"),
                clause: None,
            })
            .unwrap();

        create_resolution(
            &store,
            &scope,
            "res_decision_without_rule",
            Some("req_decided_no_rule"),
        );
        create_resolution(&store, &scope, "res_orphan", None);
        store
            .create_rule(CreateRuleInput {
                scope_id: scope.clone(),
                id: sid("rule_orphan"),
                rule_code: "ORPHAN-001".to_string(),
                name: None,
                description: None,
                requirement_id: None,
                resolution_id: None,
                statement: "An unattached rule exists".to_string(),
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
            })
            .unwrap();
        store
            .create_edge(CreateEdgeInput {
                scope_id: scope.clone(),
                edge_type: EdgeType::Contradicts,
                from_type: NodeType::Requirement,
                from_id: sid("req_contradicts_a"),
                to_type: NodeType::Requirement,
                to_id: sid("req_contradicts_b"),
            })
            .unwrap();
        store
            .create_topic(CreateTopicInput {
                scope_id: scope.clone(),
                id: sid("topic_frontier"),
                requirement_id: sid("req_question_topic"),
                title: "Frontier topic".to_string(),
                status: TopicStatus::Open,
                links: Vec::new(),
            })
            .unwrap();
        store
            .create_question(CreateQuestionInput {
                scope_id: scope.clone(),
                id: sid("question_frontier"),
                topic_id: sid("topic_frontier"),
                question: "Which path should this take?".to_string(),
                resolution_method: ResolutionMethod::Grill,
                status: QuestionStatus::Open,
                answer: None,
                links: Vec::new(),
                resolution_id: None,
            })
            .unwrap();

        let gaps = find_gaps(&layout, &scope).unwrap();
        assert_gap(
            &gaps,
            GapKind::MissingSourceRefs,
            NodeType::Requirement,
            "req_missing_source",
        );
        assert_gap(
            &gaps,
            GapKind::NoResolvingDecision,
            NodeType::Requirement,
            "req_resolved_no_decision",
        );
        assert_gap(
            &gaps,
            GapKind::NoProducedRules,
            NodeType::Requirement,
            "req_decided_no_rule",
        );
        assert_gap(
            &gaps,
            GapKind::OrphanResolution,
            NodeType::Resolution,
            "res_orphan",
        );
        assert_gap(&gaps, GapKind::OrphanRule, NodeType::Rule, "rule_orphan");
        assert_gap(
            &gaps,
            GapKind::UnreferencedSource,
            NodeType::Source,
            "source_unused",
        );
        assert_gap(
            &gaps,
            GapKind::DanglingReference,
            NodeType::Source,
            "source_dangling",
        );
        assert_gap(
            &gaps,
            GapKind::UnresolvedContradictsPair,
            NodeType::Requirement,
            "req_contradicts_a",
        );
        assert_gap(
            &gaps,
            GapKind::OpenQuestion,
            NodeType::Question,
            "question_frontier",
        );
        assert_gap(
            &gaps,
            GapKind::UnexploredTopic,
            NodeType::Topic,
            "topic_frontier",
        );
        let contradiction = gaps
            .iter()
            .find(|gap| gap.kind == GapKind::UnresolvedContradictsPair)
            .unwrap();
        assert_eq!(contradiction.related_node_type, Some(NodeType::Requirement));
        assert_eq!(
            contradiction.related_node_id.as_deref(),
            Some("req_contradicts_b")
        );

        let prime = prime_context(&layout, &scope, false).unwrap();
        assert_eq!(prime.gaps, gaps);
    }

    #[test]
    fn prime_renders_frontier_gap_subjects() {
        let (_dir, layout, scope) = seeded_layout();
        let store = StateStore::new(layout.clone());
        create_source(&store, &scope, "source_anchor");
        create_requirement(
            &store,
            &scope,
            "req_contradicts_a",
            RequirementStatus::Active,
        );
        create_requirement(
            &store,
            &scope,
            "req_contradicts_b",
            RequirementStatus::Active,
        );
        create_requirement(
            &store,
            &scope,
            "req_question_topic",
            RequirementStatus::Active,
        );
        attach_source(&store, &scope, "req_contradicts_a");
        attach_source(&store, &scope, "req_contradicts_b");
        attach_source(&store, &scope, "req_question_topic");
        store
            .create_edge(CreateEdgeInput {
                scope_id: scope.clone(),
                edge_type: EdgeType::Contradicts,
                from_type: NodeType::Requirement,
                from_id: sid("req_contradicts_a"),
                to_type: NodeType::Requirement,
                to_id: sid("req_contradicts_b"),
            })
            .unwrap();
        store
            .create_topic(CreateTopicInput {
                scope_id: scope.clone(),
                id: sid("topic_frontier"),
                requirement_id: sid("req_question_topic"),
                title: "Frontier topic".to_string(),
                status: TopicStatus::Open,
                links: Vec::new(),
            })
            .unwrap();
        store
            .create_question(CreateQuestionInput {
                scope_id: scope.clone(),
                id: sid("question_frontier"),
                topic_id: sid("topic_frontier"),
                question: "Which path should this take?".to_string(),
                resolution_method: ResolutionMethod::Grill,
                status: QuestionStatus::Open,
                answer: None,
                links: Vec::new(),
                resolution_id: None,
            })
            .unwrap();

        let view = prime_context(&layout, &scope, false).unwrap();
        let rendered = render_prime_markdown(&view);

        assert!(rendered.contains(
            "- requirement req_contradicts_a -> requirement req_contradicts_b: unresolved `contradicts` pair"
        ));
        assert!(rendered.contains("- question question_frontier: open question"));
        assert!(rendered.contains("- topic topic_frontier: unexplored topic"));
    }
}
