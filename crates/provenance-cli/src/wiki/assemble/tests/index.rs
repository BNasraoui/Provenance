use super::super::build_corpus;
use super::fixtures::*;
use crate::wiki::links::LinkResolver;
use crate::wiki::model::{CorpusCounts, GapKind};
use provenance_core::{NodeType, QuestionStatus, RequirementStatus, TopicStatus};

#[test]
fn build_corpus_on_a_truly_empty_scope_is_honestly_empty() {
    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&empty_state(), &resolver);
    assert!(corpus.requirements.is_empty());
    assert!(corpus.resolutions.is_empty());
    assert!(corpus.rules.is_empty());
    assert!(corpus.sources.is_empty());
    assert!(corpus.index.roots.is_empty());
    assert!(corpus.index.gaps.is_empty());
    assert!(corpus.index.orphans.is_empty());
    assert_eq!(corpus.index.counts, CorpusCounts::default());
}

#[test]
fn index_lists_root_requirements_with_counts() {
    let corpus = fixture_corpus();
    let roots: Vec<&str> = corpus
        .index
        .roots
        .iter()
        .map(|entry| entry.link.target.record_id.as_str())
        .collect();
    assert_eq!(roots, vec!["req_root", "req_stuck"]);
    let root = &corpus.index.roots[0];
    assert_eq!(root.children, 1);
    assert_eq!(root.resolutions, 0);
    assert_eq!(root.rules, 0);
    assert_eq!(corpus.index.counts.sources, 2);
    assert_eq!(corpus.index.counts.requirements, 3);
    assert_eq!(corpus.index.counts.resolutions, 2);
    assert_eq!(corpus.index.counts.rules, 2);
}

#[test]
fn index_reports_scope_gaps_and_orphans() {
    let corpus = fixture_corpus();
    let kinds = gap_kinds(&corpus.index.gaps);
    assert_eq!(
        kinds,
        vec![
            GapKind::MissingSourceRefs,
            GapKind::MissingSourceRefs,
            GapKind::NoResolvingDecision,
            GapKind::NoProducedRules,
            GapKind::NoProducedRules,
            GapKind::DanglingReference,
        ]
    );
    assert!(corpus
        .index
        .gaps
        .iter()
        .any(|gap| gap.detail.contains("req_root")));
    assert!(corpus
        .index
        .gaps
        .iter()
        .any(|gap| gap.detail.contains("req_stuck")));
    assert!(corpus
        .index
        .gaps
        .iter()
        .any(|gap| gap.detail.contains("res_orphan")));
    assert!(corpus
        .index
        .gaps
        .iter()
        .any(|gap| gap.detail.contains("source_missing")));
    let orphan_ids = |links: &[crate::wiki::model::PageLink]| {
        links
            .iter()
            .map(|link| link.target.record_id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(orphan_ids(&corpus.index.orphans.rules), vec!["rule_orphan"]);
    assert_eq!(
        orphan_ids(&corpus.index.orphans.resolutions),
        vec!["res_orphan"]
    );
    assert_eq!(
        orphan_ids(&corpus.index.orphans.sources),
        vec!["source_unused"]
    );
}

#[test]
fn index_filters_question_and_topic_frontier_gaps_only() {
    let mut state = empty_state();
    state.requirements = vec![requirement(
        "req_frontier",
        "Platform shall settle frontier questions",
        RequirementStatus::Active,
        vec![],
    )];
    state.topics = vec![topic("topic_open", "req_frontier", TopicStatus::Open)];
    state.questions = vec![question(
        "question_open",
        "topic_open",
        "req_frontier",
        QuestionStatus::Open,
    )];
    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&state, &resolver);

    let index_kinds = gap_kinds(&corpus.index.gaps);
    assert!(!index_kinds.contains(&GapKind::OpenQuestion));
    assert!(!index_kinds.contains(&GapKind::UnexploredTopic));

    let all_gap_kinds: Vec<GapKind> = compute_state_gaps(&state)
        .iter()
        .map(|gap| gap.kind)
        .collect();
    assert!(all_gap_kinds.contains(&GapKind::OpenQuestion));
    assert!(all_gap_kinds.contains(&GapKind::UnexploredTopic));
}

#[test]
fn index_reports_a_gap_for_a_thread_whose_parent_record_is_gone() {
    // A thread whose parent has been deleted/renamed is never matched
    // by any page's threads_for() lookup (those only ever query ids of
    // records that were found), so it would otherwise be dropped
    // without a trace instead of becoming a gap notice like every
    // other kind of dangling reference.
    let mut state = empty_state();
    state.requirements = vec![requirement(
        "req_child",
        "SaveInvoice shall split claim items",
        RequirementStatus::Active,
        vec![],
    )];
    state.threads = vec![thread(
        "thr_ghost",
        (NodeType::Resolution, "res_missing"),
        10,
    )];
    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&state, &resolver);
    let dangling = corpus
        .index
        .gaps
        .iter()
        .find(|gap| gap.kind == GapKind::DanglingReference)
        .expect("a dangling thread parent should be reported as a gap");
    assert!(dangling.detail.contains("thr_ghost"));
    assert!(dangling.detail.contains("res_missing"));
}
