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
    assert_eq!(corpus.index.title, "Default documentation");
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
    for id in ["req_root", "req_stuck", "res_orphan"] {
        assert!(corpus.index.gaps.iter().any(|gap| {
            gap.subject
                .as_ref()
                .is_some_and(|subject| subject.target.record_id == id)
        }));
    }
    assert!(corpus
        .index
        .gaps
        .iter()
        .any(|gap| gap.detail.contains("source that is missing")));
    let orphan_ids = |links: &[crate::wiki::model::PageLink]| {
        links
            .iter()
            .map(|link| link.target.record_id.clone())
            .collect::<Vec<_>>()
    };
    let orphan_rules: Vec<crate::wiki::model::PageLink> = corpus
        .index
        .orphans
        .rules
        .iter()
        .map(|rule| rule.link.clone())
        .collect();
    assert_eq!(orphan_ids(&orphan_rules), vec!["rule_orphan"]);
    assert_eq!(
        corpus.index.orphans.rules[0].reason,
        "no requirement or resolution produces this rule"
    );
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
    assert_eq!(
        dangling.detail,
        "A discussion belongs to a decision that is missing."
    );
    assert!(dangling.subject.is_none());
}

#[test]
fn findings_page_preserves_every_computed_gap_exactly_once() {
    let state = fixture_state();
    let expected = compute_state_gaps(&state)
        .into_iter()
        .map(|gap| gap.kind)
        .collect::<Vec<_>>();
    let corpus = build_corpus(&state, &LinkResolver::new(None));
    let actual = corpus
        .findings
        .findings
        .iter()
        .map(|gap| gap.kind)
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
    assert_eq!(corpus.index.finding_count, actual.len());
}

#[test]
fn findings_use_plain_sentences_and_link_existing_record_titles() {
    let corpus = fixture_corpus();
    let html = crate::wiki::render::render_findings("default", &corpus.findings);

    assert!(
        html.contains(
            "<a href=\"/requirements/req_root/\">Platform shall manage invoicing</a> has no source references."
        ),
        "{html}"
    );
    assert!(
        html.contains(
            "<a href=\"/rules/rule_orphan/\">Rule orphan</a> has no producing requirement or decision."
        ),
        "{html}"
    );
    assert!(!html.contains('`'), "{html}");
    assert!(!html.contains("requirement req_root:"), "{html}");
}
