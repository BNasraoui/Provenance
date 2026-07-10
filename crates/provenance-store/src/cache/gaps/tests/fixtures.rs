use super::super::{compute_gaps, GapGraph, GapItem, GapKind};
use provenance_core::{
    Edge, EdgeType, NodeType, Question, QuestionStatus, Requirement, RequirementStatus, Resolution,
    ResolutionMethod, ResolutionStatus, Rule, RuleSeverity, RuleStatus, SchemaVersion, ScopeId,
    Source, SourceType, StableId, Topic, TopicStatus,
};

pub fn sid(value: &str) -> StableId {
    StableId::new(value).unwrap()
}

fn scope_id() -> ScopeId {
    ScopeId::new("default").unwrap()
}

pub fn requirement(id: &str) -> Requirement {
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

pub fn resolution(id: &str) -> Resolution {
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

pub fn rule(id: &str) -> Rule {
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

pub fn topic(id: &str, status: TopicStatus) -> Topic {
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

pub fn question(id: &str, topic_id: &str, status: QuestionStatus) -> Question {
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

pub fn source(id: &str) -> Source {
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

pub fn edge(edge_type: EdgeType, from: (NodeType, &str), to: (NodeType, &str)) -> Edge {
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

pub fn compute_for(
    sources: &[Source],
    requirements: &[Requirement],
    resolutions: &[Resolution],
    rules: &[Rule],
    topics: &[Topic],
    questions: &[Question],
    edges: &[Edge],
) -> Vec<GapItem> {
    let scope = scope_id();
    compute_gaps(&GapGraph {
        scope: &scope,
        sources,
        requirements,
        resolutions,
        rules,
        topics,
        questions,
        edges,
        threads: &[],
    })
}

pub fn count_kind(gaps: &[GapItem], kind: GapKind) -> usize {
    gaps.iter().filter(|gap| gap.kind == kind).count()
}
