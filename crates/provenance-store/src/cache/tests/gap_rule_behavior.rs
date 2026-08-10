//! The gap rule: `compute_gaps` decides what counts as an unfinished spot
//! in a graph. Proven per kind by minimal graphs, and by properties over
//! complete graphs.

use super::super::*;
use super::fixtures::*;
use crate::cache::gaps::tests::fixtures as records;
use provenance_core::{
    Edge, EdgeType, NodeType, Question, QuestionStatus, Requirement, RequirementStatus, Resolution,
    ResolutionStatus, Rule, Source, SourceReference, Topic, TopicStatus,
};
use provenance_macros::verifies;

const ANCHOR_SOURCE: &str = "source_anchor";
const ANCHOR_DOMAIN: &str = "domain_shaping";

/// The variant list is derived from an exhaustive match so that adding a
/// `GapKind` fails compilation until the new variant joins the chain, which
/// keeps the per-kind exhaustion below complete.
pub(super) fn all_gap_kinds() -> Vec<GapKind> {
    let mut all = vec![GapKind::MissingDomainId];
    while let Some(next) = match all.last().unwrap() {
        GapKind::MissingDomainId => Some(GapKind::MissingSourceRefs),
        GapKind::MissingSourceRefs => Some(GapKind::NoResolvingDecision),
        GapKind::NoResolvingDecision => Some(GapKind::NoProducedRules),
        GapKind::NoProducedRules => Some(GapKind::OrphanRule),
        GapKind::OrphanRule => Some(GapKind::OrphanResolution),
        GapKind::OrphanResolution => Some(GapKind::UnreferencedSource),
        GapKind::UnreferencedSource => Some(GapKind::DanglingReference),
        GapKind::DanglingReference => Some(GapKind::UnresolvedContradictsPair),
        GapKind::UnresolvedContradictsPair => Some(GapKind::OpenQuestion),
        GapKind::OpenQuestion => Some(GapKind::UnexploredTopic),
        GapKind::UnexploredTopic => None,
    } {
        all.push(next);
    }
    all
}

/// Owned records fed straight to `compute_gaps`, bypassing the state store so
/// a case can hold exactly the records it means to hold and nothing else.
#[derive(Default)]
struct HandBuiltGraph {
    sources: Vec<Source>,
    requirements: Vec<Requirement>,
    resolutions: Vec<Resolution>,
    rules: Vec<Rule>,
    topics: Vec<Topic>,
    questions: Vec<Question>,
    edges: Vec<Edge>,
}

impl HandBuiltGraph {
    fn gaps(&self) -> Vec<GapItem> {
        records::compute_for(
            &self.sources,
            &self.requirements,
            &self.resolutions,
            &self.rules,
            &self.topics,
            &self.questions,
            &self.edges,
        )
    }
}

/// A requirement carrying a source ref but no domain.
fn sourced_requirement(id: &str, source_id: &str, status: RequirementStatus) -> Requirement {
    Requirement {
        status,
        source_refs: vec![SourceReference {
            source_id: sid(source_id),
            clause: None,
        }],
        ..records::requirement(id)
    }
}

/// A requirement with nothing missing on its own account: domain assigned and
/// a live source behind it.
fn settled_requirement(id: &str, source_id: &str, status: RequirementStatus) -> Requirement {
    Requirement {
        domain_id: Some(sid(ANCHOR_DOMAIN)),
        ..sourced_requirement(id, source_id, status)
    }
}

/// One minimal graph per gap kind. The match is exhaustive, so a new
/// `GapKind` cannot be added without stating the smallest graph that opens it.
#[allow(clippy::too_many_lines)]
fn minimal_graph_for(kind: GapKind) -> HandBuiltGraph {
    match kind {
        GapKind::MissingDomainId => HandBuiltGraph {
            sources: vec![records::source(ANCHOR_SOURCE)],
            requirements: vec![sourced_requirement(
                "req_no_domain",
                ANCHOR_SOURCE,
                RequirementStatus::Active,
            )],
            ..HandBuiltGraph::default()
        },
        GapKind::MissingSourceRefs => HandBuiltGraph {
            requirements: vec![Requirement {
                domain_id: Some(sid(ANCHOR_DOMAIN)),
                ..records::requirement("req_no_source")
            }],
            ..HandBuiltGraph::default()
        },
        // Resolved, and a rule of its own, but nothing recorded as having
        // decided it. The rule still needs a decision behind it or it would
        // be orphaned as well, and that decision has to settle some other
        // requirement or it would be orphaned in turn, which is why the
        // smallest graph for this one kind runs to a second chain.
        GapKind::NoResolvingDecision => HandBuiltGraph {
            sources: vec![records::source(ANCHOR_SOURCE)],
            requirements: vec![
                settled_requirement("req_resolved", ANCHOR_SOURCE, RequirementStatus::Resolved),
                settled_requirement("req_decided", ANCHOR_SOURCE, RequirementStatus::Active),
            ],
            resolutions: vec![records::resolution("res_decided")],
            rules: vec![records::rule("rule_direct")],
            edges: vec![
                records::edge(
                    EdgeType::Produces,
                    (NodeType::Requirement, "req_resolved"),
                    (NodeType::Rule, "rule_direct"),
                ),
                records::edge(
                    EdgeType::Produces,
                    (NodeType::Resolution, "res_decided"),
                    (NodeType::Rule, "rule_direct"),
                ),
                records::edge(
                    EdgeType::Resolves,
                    (NodeType::Resolution, "res_decided"),
                    (NodeType::Requirement, "req_decided"),
                ),
            ],
            ..HandBuiltGraph::default()
        },
        // Decided, but the decision produced nothing. The decision itself is
        // still a draft, so it is not separately on the hook for a rule.
        GapKind::NoProducedRules => HandBuiltGraph {
            sources: vec![records::source(ANCHOR_SOURCE)],
            requirements: vec![settled_requirement(
                "req_decided",
                ANCHOR_SOURCE,
                RequirementStatus::Active,
            )],
            resolutions: vec![records::resolution("res_decided")],
            edges: vec![records::edge(
                EdgeType::Resolves,
                (NodeType::Resolution, "res_decided"),
                (NodeType::Requirement, "req_decided"),
            )],
            ..HandBuiltGraph::default()
        },
        // A chain written only halfway: the requirement is recorded as
        // producing the rule, but the decision that settled it is not, so
        // the rule is not traced back to the decision behind it. The
        // decision is still a draft, so it is not separately on the hook
        // for a rule.
        GapKind::OrphanRule => HandBuiltGraph {
            sources: vec![records::source(ANCHOR_SOURCE)],
            requirements: vec![settled_requirement(
                "req_half_traced",
                ANCHOR_SOURCE,
                RequirementStatus::Active,
            )],
            resolutions: vec![records::resolution("res_half_traced")],
            rules: vec![records::rule("rule_half_traced")],
            edges: vec![
                records::edge(
                    EdgeType::Resolves,
                    (NodeType::Resolution, "res_half_traced"),
                    (NodeType::Requirement, "req_half_traced"),
                ),
                records::edge(
                    EdgeType::Produces,
                    (NodeType::Requirement, "req_half_traced"),
                    (NodeType::Rule, "rule_half_traced"),
                ),
            ],
            ..HandBuiltGraph::default()
        },
        GapKind::OrphanResolution => HandBuiltGraph {
            resolutions: vec![records::resolution("res_orphan")],
            ..HandBuiltGraph::default()
        },
        GapKind::UnreferencedSource => HandBuiltGraph {
            sources: vec![records::source("source_unused")],
            ..HandBuiltGraph::default()
        },
        // An explored topic hung off a requirement that is not there.
        GapKind::DanglingReference => HandBuiltGraph {
            topics: vec![records::topic_for(
                "topic_dangling",
                "req_missing",
                TopicStatus::Explored,
            )],
            ..HandBuiltGraph::default()
        },
        GapKind::UnresolvedContradictsPair => HandBuiltGraph {
            sources: vec![records::source(ANCHOR_SOURCE)],
            requirements: vec![
                settled_requirement("req_left", ANCHOR_SOURCE, RequirementStatus::Active),
                settled_requirement("req_right", ANCHOR_SOURCE, RequirementStatus::Active),
            ],
            edges: vec![records::edge(
                EdgeType::Contradicts,
                (NodeType::Requirement, "req_left"),
                (NodeType::Requirement, "req_right"),
            )],
            ..HandBuiltGraph::default()
        },
        GapKind::OpenQuestion => HandBuiltGraph {
            sources: vec![records::source(ANCHOR_SOURCE)],
            requirements: vec![settled_requirement(
                "req_asked",
                ANCHOR_SOURCE,
                RequirementStatus::Active,
            )],
            topics: vec![records::topic_for(
                "topic_asked",
                "req_asked",
                TopicStatus::Explored,
            )],
            questions: vec![records::question_for(
                "question_open",
                "topic_asked",
                "req_asked",
                QuestionStatus::Open,
            )],
            ..HandBuiltGraph::default()
        },
        GapKind::UnexploredTopic => HandBuiltGraph {
            sources: vec![records::source(ANCHOR_SOURCE)],
            requirements: vec![settled_requirement(
                "req_open_topic",
                ANCHOR_SOURCE,
                RequirementStatus::Active,
            )],
            topics: vec![records::topic_for(
                "topic_open",
                "req_open_topic",
                TopicStatus::Open,
            )],
            ..HandBuiltGraph::default()
        },
    }
}

/// A graph with nothing left unfinished: every requirement has a domain, a
/// live source, a decision that resolves it, and a rule that both the
/// requirement and the decision are recorded as producing; every topic is
/// explored and every question answered.
///
/// The two `produces` spellings used to be alternatives, and the properties
/// below ran once per spelling. A rule now counts as traced only when both
/// edges are there, so a complete chain carries both and the alternatives
/// are gone.
fn complete_graph(chains: usize) -> HandBuiltGraph {
    let mut graph = HandBuiltGraph::default();
    for index in 0..chains {
        let source = format!("source_{index}");
        let requirement = format!("req_{index}");
        let resolution = format!("res_{index}");
        let rule = format!("rule_{index}");
        let topic = format!("topic_{index}");
        let question = format!("question_{index}");
        graph.sources.push(records::source(&source));
        graph.requirements.push(settled_requirement(
            &requirement,
            &source,
            RequirementStatus::Resolved,
        ));
        graph.resolutions.push(Resolution {
            status: ResolutionStatus::Approved,
            ..records::resolution(&resolution)
        });
        graph.rules.push(records::rule(&rule));
        graph.topics.push(records::topic_for(
            &topic,
            &requirement,
            TopicStatus::Explored,
        ));
        graph.questions.push(records::question_for(
            &question,
            &topic,
            &requirement,
            QuestionStatus::Answered,
        ));
        graph.edges.push(records::edge(
            EdgeType::Resolves,
            (NodeType::Resolution, &resolution),
            (NodeType::Requirement, &requirement),
        ));
        graph.edges.push(records::edge(
            EdgeType::Produces,
            (NodeType::Resolution, &resolution),
            (NodeType::Rule, &rule),
        ));
        graph.edges.push(records::edge(
            EdgeType::Produces,
            (NodeType::Requirement, &requirement),
            (NodeType::Rule, &rule),
        ));
    }
    graph
}

/// The reason recorded against a rule's orphan gap, if it has one.
fn orphan_rule_reason(graph: &HandBuiltGraph, rule_id: &str) -> Option<String> {
    graph
        .gaps()
        .into_iter()
        .find(|gap| gap.kind == GapKind::OrphanRule && gap.node_id == rule_id)
        .map(|gap| gap.reason)
}

#[test]
#[verifies("rule_graph_gaps", exhaustion)]
fn every_gap_kind_has_a_minimal_graph_that_produces_exactly_it() {
    for kind in all_gap_kinds() {
        let gaps = minimal_graph_for(kind).gaps();
        assert_eq!(
            gaps.iter().map(|gap| gap.kind).collect::<Vec<_>>(),
            vec![kind],
            "the minimal graph for {kind:?} does not produce exactly that gap; got {gaps:#?}"
        );
    }
}

#[test]
#[verifies("rule_graph_gaps", property)]
fn a_complete_graph_has_no_gaps() {
    for chains in 0..=6 {
        let graph = complete_graph(chains);
        let gaps = graph.gaps();
        assert!(
            gaps.is_empty(),
            "a complete graph of {chains} chains reports gaps: {gaps:#?}"
        );
    }
}

#[test]
#[verifies("rule_graph_gaps", property)]
fn removing_any_single_link_from_a_complete_graph_opens_a_gap() {
    for chains in 1..=4 {
        let complete = complete_graph(chains);
        for dropped in 0..complete.edges.len() {
            let mut graph = complete_graph(chains);
            let removed = graph.edges.remove(dropped);
            let gaps = graph.gaps();
            assert!(
                !gaps.is_empty(),
                "dropping {:?} edge {} from a complete graph of {chains} chains left no gap",
                removed.edge_type,
                removed.id.as_str()
            );
        }
        for orphaned in 0..chains {
            let mut graph = complete_graph(chains);
            graph.requirements[orphaned].source_refs.clear();
            let gaps = graph.gaps();
            assert!(
                !gaps.is_empty(),
                "dropping the source ref of requirement {orphaned} from a complete graph \
                 of {chains} chains left no gap"
            );
        }
    }
}

/// The owner's ruling on two definitions that shipped at once: a rule is
/// traced only when both ends produce it. Either `produces` edge alone
/// leaves the gap open, and the gap says which end is missing so a reader
/// knows what to add.
#[test]
#[verifies("rule_graph_gaps", examples)]
fn a_rule_produced_from_one_end_only_stays_orphaned_and_the_gap_names_the_missing_end() {
    let mut requirement_only = complete_graph(1);
    requirement_only.edges.retain(|edge| {
        !(edge.edge_type == EdgeType::Produces && edge.from_type == NodeType::Resolution)
    });
    assert_eq!(
        orphan_rule_reason(&requirement_only, "rule_0").as_deref(),
        Some("no resolution produces this rule")
    );

    let mut resolution_only = complete_graph(1);
    resolution_only.edges.retain(|edge| {
        !(edge.edge_type == EdgeType::Produces && edge.from_type == NodeType::Requirement)
    });
    assert_eq!(
        orphan_rule_reason(&resolution_only, "rule_0").as_deref(),
        Some("no requirement produces this rule")
    );

    let unattached = HandBuiltGraph {
        rules: vec![records::rule("rule_unattached")],
        ..HandBuiltGraph::default()
    };
    assert_eq!(
        orphan_rule_reason(&unattached, "rule_unattached").as_deref(),
        Some("no requirement or resolution produces this rule")
    );

    assert_eq!(orphan_rule_reason(&complete_graph(1), "rule_0"), None);
}
