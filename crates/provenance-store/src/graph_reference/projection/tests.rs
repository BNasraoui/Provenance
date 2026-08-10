use provenance_core::{
    EdgeType, NodeType, QuestionStatus, RepoPathPrefix, RequirementStatus, ResolutionMethod,
    ResolutionStatus, RuleSeverity, RuleStatus, SchemaVersion, ServiceBindingType, ServiceStatus,
    SourceType, StableId, TopicStatus,
};
use provenance_macros::verifies;

use super::*;

mod collaboration;

/// The record families the ownership check walks. One variant per
/// `require_scope!` line, and one per record-bearing field of
/// `GraphExport`; `family_count_of` below ties the two together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordFamily {
    Source,
    Domain,
    Requirement,
    Boundary,
    Topic,
    Question,
    Resolution,
    Rule,
    Service,
    ServiceBinding,
    Edge,
}

// Built from an exhaustive match so that adding a RecordFamily variant
// fails compilation until the new family joins the chain, keeping the
// exhaustion proof below complete.
fn all_families() -> Vec<RecordFamily> {
    let mut all = vec![RecordFamily::Source];
    while let Some(next) = match all.last().unwrap() {
        RecordFamily::Source => Some(RecordFamily::Domain),
        RecordFamily::Domain => Some(RecordFamily::Requirement),
        RecordFamily::Requirement => Some(RecordFamily::Boundary),
        RecordFamily::Boundary => Some(RecordFamily::Topic),
        RecordFamily::Topic => Some(RecordFamily::Question),
        RecordFamily::Question => Some(RecordFamily::Resolution),
        RecordFamily::Resolution => Some(RecordFamily::Rule),
        RecordFamily::Rule => Some(RecordFamily::Service),
        RecordFamily::Service => Some(RecordFamily::ServiceBinding),
        RecordFamily::ServiceBinding => Some(RecordFamily::Edge),
        RecordFamily::Edge => None,
    } {
        all.push(next);
    }
    all
}

const fn record_id(family: RecordFamily) -> &'static str {
    match family {
        RecordFamily::Source => "source_pinned",
        RecordFamily::Domain => "domain_pinned",
        RecordFamily::Requirement => "req_pinned",
        RecordFamily::Boundary => "boundary_pinned",
        RecordFamily::Topic => "topic_pinned",
        RecordFamily::Question => "question_pinned",
        RecordFamily::Resolution => "res_pinned",
        RecordFamily::Rule => "rule_pinned",
        RecordFamily::Service => "service_pinned",
        RecordFamily::ServiceBinding => "service_binding_pinned",
        RecordFamily::Edge => "edge_pinned",
    }
}

fn stable_id(family: RecordFamily) -> StableId {
    StableId::new(record_id(family)).unwrap()
}

fn source_record(scope: &ScopeId) -> Source {
    Source {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Source),
        name: "Pinned source".into(),
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

fn domain_record(scope: &ScopeId) -> Domain {
    Domain {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Domain),
        name: "Pinned domain".into(),
        description: None,
        color: None,
    }
}

fn requirement_record(scope: &ScopeId) -> Requirement {
    Requirement {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Requirement),
        statement: "Pinned requirement".into(),
        description: None,
        fog: None,
        status: RequirementStatus::Active,
        domain_id: None,
        source_refs: Vec::new(),
        origin_thread: None,
        origin_message: None,
    }
}

fn boundary_record(scope: &ScopeId) -> Boundary {
    Boundary {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Boundary),
        requirement_id: stable_id(RecordFamily::Requirement),
        statement: "Pinned boundary".into(),
        source_ref: None,
    }
}

fn topic_record(scope: &ScopeId) -> Topic {
    Topic {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Topic),
        requirement_id: stable_id(RecordFamily::Requirement),
        title: "Pinned topic".into(),
        status: TopicStatus::Open,
        claimed_by: None,
        claimed_at: None,
        links: Vec::new(),
    }
}

fn question_record(scope: &ScopeId) -> Question {
    Question {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Question),
        topic_id: stable_id(RecordFamily::Topic),
        requirement_id: stable_id(RecordFamily::Requirement),
        question: "Pinned question?".into(),
        resolution_method: ResolutionMethod::Grill,
        status: QuestionStatus::Open,
        claimed_by: None,
        claimed_at: None,
        answer: None,
        links: Vec::new(),
        resolution_id: None,
    }
}

fn resolution_record(scope: &ScopeId) -> Resolution {
    Resolution {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Resolution),
        title: "Pinned resolution".into(),
        position: "Pinned position".into(),
        rationale: "Pinned rationale".into(),
        status: ResolutionStatus::Approved,
        context: None,
        enforcement: None,
        confidence: None,
        inputs: Vec::new(),
        made_by: None,
        approved_by: None,
        approved_at: None,
        superseded_by: None,
        review_on: None,
        origin_thread: None,
        origin_message: None,
    }
}

fn rule_record(scope: &ScopeId) -> Rule {
    Rule {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Rule),
        rule_code: "PINNED-1".into(),
        name: None,
        description: None,
        statement: "Pinned rule".into(),
        status: RuleStatus::Active,
        severity: RuleSeverity::Medium,
        rule_type: None,
        modality: None,
        confidence: None,
        extraction_method: None,
        source_document: None,
        source_section: None,
        origin_thread: None,
        origin_message: None,
    }
}

fn service_record(scope: &ScopeId) -> Service {
    Service {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Service),
        name: "Pinned service".into(),
        description: None,
        owner: None,
        repository: None,
        environment: None,
        tier: None,
        external_id: None,
        status: ServiceStatus::Active,
    }
}

fn service_binding_record(scope: &ScopeId) -> ServiceBinding {
    ServiceBinding {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::ServiceBinding),
        rule_id: stable_id(RecordFamily::Rule),
        service_id: stable_id(RecordFamily::Service),
        binding_type: ServiceBindingType::Enforces,
    }
}

fn edge_record(scope: &ScopeId) -> Edge {
    Edge {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: stable_id(RecordFamily::Edge),
        edge_type: EdgeType::References,
        from_type: NodeType::Source,
        from_id: stable_id(RecordFamily::Source),
        to_type: NodeType::Requirement,
        to_id: stable_id(RecordFamily::Requirement),
        label: None,
    }
}

/// A graph claiming `scope`, holding exactly one record of each named
/// family and nothing else. Every record sits in the claimed scope.
fn graph_in_scope(scope: &ScopeId, populated: &[RecordFamily]) -> GraphExport {
    let holds = |family: RecordFamily| populated.contains(&family);
    GraphExport {
        schema_version: 1,
        scope: Scope {
            id: scope.clone(),
            path_prefix: RepoPathPrefix::new("."),
        },
        sources: holds(RecordFamily::Source)
            .then(|| source_record(scope))
            .into_iter()
            .collect(),
        domains: holds(RecordFamily::Domain)
            .then(|| domain_record(scope))
            .into_iter()
            .collect(),
        requirements: holds(RecordFamily::Requirement)
            .then(|| requirement_record(scope))
            .into_iter()
            .collect(),
        boundaries: holds(RecordFamily::Boundary)
            .then(|| boundary_record(scope))
            .into_iter()
            .collect(),
        topics: holds(RecordFamily::Topic)
            .then(|| topic_record(scope))
            .into_iter()
            .collect(),
        questions: holds(RecordFamily::Question)
            .then(|| question_record(scope))
            .into_iter()
            .collect(),
        resolutions: holds(RecordFamily::Resolution)
            .then(|| resolution_record(scope))
            .into_iter()
            .collect(),
        rules: holds(RecordFamily::Rule)
            .then(|| rule_record(scope))
            .into_iter()
            .collect(),
        services: holds(RecordFamily::Service)
            .then(|| service_record(scope))
            .into_iter()
            .collect(),
        service_bindings: holds(RecordFamily::ServiceBinding)
            .then(|| service_binding_record(scope))
            .into_iter()
            .collect(),
        edges: holds(RecordFamily::Edge)
            .then(|| edge_record(scope))
            .into_iter()
            .collect(),
    }
}

/// Moves every record of one family into another scope, leaving the rest
/// of the graph alone.
fn move_family_to_scope(graph: &mut GraphExport, family: RecordFamily, scope: &ScopeId) {
    match family {
        RecordFamily::Source => {
            for record in &mut graph.sources {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Domain => {
            for record in &mut graph.domains {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Requirement => {
            for record in &mut graph.requirements {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Boundary => {
            for record in &mut graph.boundaries {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Topic => {
            for record in &mut graph.topics {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Question => {
            for record in &mut graph.questions {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Resolution => {
            for record in &mut graph.resolutions {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Rule => {
            for record in &mut graph.rules {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Service => {
            for record in &mut graph.services {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::ServiceBinding => {
            for record in &mut graph.service_bindings {
                record.scope_id = scope.clone();
            }
        }
        RecordFamily::Edge => {
            for record in &mut graph.edges {
                record.scope_id = scope.clone();
            }
        }
    }
}

/// Counts the record families a graph carries, by destructuring
/// `GraphExport` exhaustively. Adding a record-bearing field to
/// `GraphExport` fails compilation here, and the count assertion in the
/// exhaustion proof then fails until the new family joins `RecordFamily`
/// and the fixture. The domain of the proof is therefore the field list
/// the `require_scope!` macro walks, not a hand-kept list.
fn family_count_of(graph: &GraphExport) -> usize {
    let GraphExport {
        schema_version: _,
        scope: _,
        sources,
        domains,
        requirements,
        boundaries,
        topics,
        questions,
        resolutions,
        rules,
        services,
        service_bindings,
        edges,
    } = graph;
    let counts = [
        sources.len(),
        domains.len(),
        requirements.len(),
        boundaries.len(),
        topics.len(),
        questions.len(),
        resolutions.len(),
        rules.len(),
        services.len(),
        service_bindings.len(),
        edges.len(),
    ];
    assert!(
        counts.iter().all(|count| *count == 1),
        "the fully populated fixture must hold exactly one record of every family, got {counts:?}"
    );
    counts.len()
}

#[test]
#[verifies("rule_pinned_scope_ownership", exhaustion)]
fn rejects_a_stray_record_of_every_family() {
    let claimed = ScopeId::new("default").unwrap();
    let elsewhere = ScopeId::new("other").unwrap();
    let families = all_families();

    assert_eq!(
        family_count_of(&graph_in_scope(&claimed, &families)),
        families.len(),
        "every record family of GraphExport must be covered by RecordFamily"
    );

    for &family in &families {
        let mut graph = graph_in_scope(&claimed, &families);
        move_family_to_scope(&mut graph, family, &elsewhere);
        let Err(GraphReferenceError::Incomplete { detail }) =
            validate_scope_ownership(&graph, &claimed)
        else {
            panic!("a pinned graph holding a {family:?} record from scope 'other' was accepted");
        };
        assert!(
            detail.contains(record_id(family)),
            "the refusal must name the offending {family:?} record, got: {detail}"
        );
        assert!(
            detail.contains("'other'") && detail.contains("'default'"),
            "the refusal must name both the record's scope and the claimed scope, got: {detail}"
        );
    }
}

#[test]
#[verifies("rule_pinned_scope_ownership", property)]
fn accepts_any_graph_whose_records_all_sit_in_the_claimed_scope() {
    // The property: a graph passes whenever no record carries a scope id
    // other than the claimed one. Stated without reference to the record
    // families, so which families are populated must not matter. The
    // generator walks every subset of the families, twice over, once per
    // claimed scope id.
    let families = all_families();
    for scope_name in ["default", "team-b"] {
        let claimed = ScopeId::new(scope_name).unwrap();
        for mask in 0u32..(1u32 << families.len()) {
            let populated: Vec<RecordFamily> = families
                .iter()
                .enumerate()
                .filter(|(index, _)| mask & (1u32 << index) != 0)
                .map(|(_, family)| *family)
                .collect();
            let graph = graph_in_scope(&claimed, &populated);
            assert!(
                validate_scope_ownership(&graph, &claimed).is_ok(),
                "scope '{scope_name}' rejected a graph whose records all sit in it: {populated:?}"
            );
        }
    }
}
