use super::super::GapKind;
use super::fixtures::*;
use provenance_core::{EdgeType, NodeType, QuestionStatus, SourceReference, TopicStatus};

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
    assert!(gaps
        .iter()
        .any(|gap| gap.kind == GapKind::OrphanRule && gap.node_id == "rule_orphaned"));
    assert!(gaps.iter().any(|gap| {
        gap.kind == GapKind::DanglingReference
            && gap.node_type == NodeType::Rule
            && gap.node_id == "rule_orphaned"
            && gap.related_node_type == Some(NodeType::Resolution)
            && gap.related_node_id.as_deref() == Some("res_missing")
            && gap.reason.contains("produces edge")
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
