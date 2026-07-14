use super::super::build_corpus;
use super::fixtures::*;
use crate::wiki::links::LinkResolver;
use crate::wiki::model::{GapKind, RecordKind};
use provenance_core::{EdgeType, NodeType, RequirementStatus};

#[test]
fn requirement_page_assembles_lineage_decision_rules_and_sources() {
    let corpus = fixture_corpus();
    let page = requirement_page(&corpus, "req_child");

    let back = page.back_link.as_ref().unwrap();
    assert_eq!(back.target.record_id, "req_root");

    let lineage: Vec<(&str, bool)> = page
        .lineage
        .iter()
        .map(|entry| (entry.link.target.record_id.as_str(), entry.is_current))
        .collect();
    assert_eq!(lineage, vec![("req_root", false), ("req_child", true)]);

    assert_eq!(page.decisions.len(), 1);
    let decision = &page.decisions[0];
    assert_eq!(decision.link.target.record_id, "res_split");
    assert_eq!(decision.link.target.kind, RecordKind::Resolution);
    assert_eq!(decision.position, "Adopt the split");
    assert_eq!(decision.inputs.len(), 1);
    assert_eq!(
        decision.inputs[0].reference.href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L59-L69")
    );

    assert_eq!(
        page.produced_rules.len(),
        1,
        "direct and via-resolution rules deduplicate"
    );
    let card = &page.produced_rules[0];
    assert_eq!(card.rule_code, "SAH-INV-001");
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.evidence[0].label, "src/UseCase.php:59-69");
    assert_eq!(
        card.evidence[0].href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L59-L69")
    );

    assert_eq!(page.sources.len(), 1);
    assert_eq!(page.sources[0].link.target.record_id, "source_schads");
    assert_eq!(page.sources[0].clause.as_deref(), Some("clause 10.3"));

    assert!(page.gaps.is_empty());
}

#[test]
fn requirement_page_borrows_decision_threads_and_annotates_bodies() {
    let corpus = fixture_corpus();
    let page = requirement_page(&corpus, "req_child");
    let thread_ids: Vec<&str> = page
        .threads
        .iter()
        .map(|thread| thread.thread_id.as_str())
        .collect();
    assert_eq!(thread_ids, vec!["thr_req_child", "thr_res_split"]);
    assert_eq!(page.threads[1].parent_type, NodeType::Resolution);
    let note = &page.threads[1].messages[0];
    assert_eq!(note.refs.len(), 2);
    assert_eq!(note.refs[0].label, "src/UseCase.php:153-156");
    assert_eq!(note.refs[1].label, "testCreateGapInvoiceOnly");
}

#[test]
fn requirement_page_flags_missing_sources() {
    let corpus = fixture_corpus();
    let page = requirement_page(&corpus, "req_root");
    assert_eq!(gap_kinds(&page.gaps), vec![GapKind::MissingSourceRefs]);
    let children: Vec<&str> = page
        .children
        .iter()
        .map(|link| link.target.record_id.as_str())
        .collect();
    assert_eq!(children, vec!["req_child"]);
}

#[test]
fn requirement_page_lists_siblings_from_all_parents_without_self_or_duplicates() {
    let mut state = empty_state();
    state.requirements = vec![
        requirement(
            "req_parent_a",
            "Parent A",
            RequirementStatus::Active,
            vec![],
        ),
        requirement(
            "req_parent_b",
            "Parent B",
            RequirementStatus::Active,
            vec![],
        ),
        requirement(
            "req_sibling_beta",
            "Sibling Beta",
            RequirementStatus::Active,
            vec![],
        ),
        requirement("req_child", "Child", RequirementStatus::Active, vec![]),
        requirement(
            "req_sibling_alpha",
            "Sibling Alpha",
            RequirementStatus::Active,
            vec![],
        ),
        requirement(
            "req_sibling_only_b",
            "Sibling Only B",
            RequirementStatus::Active,
            vec![],
        ),
    ];
    state.edges = vec![
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_parent_a"),
            (NodeType::Requirement, "req_child"),
        ),
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_parent_a"),
            (NodeType::Requirement, "req_sibling_beta"),
        ),
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_parent_a"),
            (NodeType::Requirement, "req_sibling_alpha"),
        ),
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_parent_b"),
            (NodeType::Requirement, "req_child"),
        ),
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_parent_b"),
            (NodeType::Requirement, "req_sibling_alpha"),
        ),
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_parent_b"),
            (NodeType::Requirement, "req_sibling_only_b"),
        ),
    ];
    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&state, &resolver);
    let page = requirement_page(&corpus, "req_child");

    let sibling_ids: Vec<&str> = page
        .siblings
        .iter()
        .map(|link| link.target.record_id.as_str())
        .collect();
    assert_eq!(
        sibling_ids,
        vec![
            "req_sibling_beta",
            "req_sibling_alpha",
            "req_sibling_only_b"
        ]
    );
}

#[test]
fn requirement_and_index_pages_flag_requirements_without_domain_id_only() {
    let mut state = empty_state();
    let mut missing_domain = requirement(
        "req_missing_domain",
        "Rostering shall be assigned to a domain",
        RequirementStatus::Active,
        vec![],
    );
    missing_domain.domain_id = None;
    state.requirements = vec![
        missing_domain,
        requirement(
            "req_with_domain",
            "Payroll shall keep its domain assignment",
            RequirementStatus::Active,
            vec![],
        ),
    ];

    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&state, &resolver);
    let missing_page = requirement_page(&corpus, "req_missing_domain");
    let with_domain_page = requirement_page(&corpus, "req_with_domain");

    assert!(missing_page
        .gaps
        .iter()
        .any(|gap| gap.detail.contains("domain_id")));
    assert!(!with_domain_page
        .gaps
        .iter()
        .any(|gap| gap.detail.contains("domain_id")));
    assert!(corpus.index.gaps.iter().any(|gap| {
        gap.detail.contains("req_missing_domain") && gap.detail.contains("domain_id")
    }));
    assert!(!corpus
        .index
        .gaps
        .iter()
        .any(|gap| { gap.detail.contains("req_with_domain") && gap.detail.contains("domain_id") }));
}

#[test]
fn requirement_page_flags_dangling_refs_and_frontier_gaps() {
    let corpus = fixture_corpus();
    let page = requirement_page(&corpus, "req_stuck");
    assert!(page.sources.is_empty());
    let kinds = gap_kinds(&page.gaps);
    assert!(kinds.contains(&GapKind::DanglingReference));
    assert!(kinds.contains(&GapKind::MissingSourceRefs));
    assert!(kinds.contains(&GapKind::NoResolvingDecision));
    assert!(kinds.contains(&GapKind::NoProducedRules));
    let dangling = page
        .gaps
        .iter()
        .find(|gap| gap.kind == GapKind::DanglingReference)
        .unwrap();
    assert!(dangling.detail.contains("source_missing"));
}

#[test]
fn requirement_and_index_pages_anchor_dangling_edges_in_both_directions() {
    let mut state = empty_state();
    state.requirements = vec![requirement(
        "req_surviving",
        "Surviving requirement endpoint",
        RequirementStatus::Active,
        vec![],
    )];
    state.edges = vec![
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_missing_from"),
            (NodeType::Requirement, "req_surviving"),
        ),
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_surviving"),
            (NodeType::Requirement, "req_missing_to"),
        ),
    ];

    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&state, &resolver);
    let page = requirement_page(&corpus, "req_surviving");

    let dangling_details: Vec<_> = page
        .gaps
        .iter()
        .filter(|gap| gap.kind == GapKind::DanglingReference)
        .map(|gap| gap.detail.as_str())
        .collect();
    assert_eq!(dangling_details.len(), 2);
    for missing_id in ["req_missing_from", "req_missing_to"] {
        assert!(dangling_details.iter().any(|detail| {
            detail.contains("req_surviving")
                && detail.contains(missing_id)
                && detail.contains("refines_into")
        }));
        assert!(corpus.index.gaps.iter().any(|gap| {
            gap.kind == GapKind::DanglingReference
                && gap.detail.contains("req_surviving")
                && gap.detail.contains(missing_id)
                && gap.detail.contains("refines_into")
        }));
    }
}

#[test]
fn requirement_page_does_not_treat_a_same_id_record_of_another_kind_as_a_resolving_decision() {
    // A Resolution and a Source share the stable id "dup_id". The only
    // Resolves edge on file is authored for the source (not the
    // resolution), so it must not be mistaken for a real resolving
    // decision just because the ids match.
    let mut state = empty_state();
    state.requirements = vec![requirement(
        "req_child",
        "SaveInvoice shall split claim items",
        RequirementStatus::Active,
        vec![],
    )];
    state.resolutions = vec![resolution("dup_id", "Decoy resolution", vec![])];
    state.sources = vec![source("dup_id", "Decoy source")];
    state.edges = vec![edge(
        EdgeType::Resolves,
        (NodeType::Source, "dup_id"),
        (NodeType::Requirement, "req_child"),
    )];
    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&state, &resolver);
    let page = requirement_page(&corpus, "req_child");
    assert!(
        page.decisions.is_empty(),
        "resolution 'dup_id' has no real Resolves edge and must not appear as a decision"
    );
}

#[test]
fn contradiction_gap_surfaces_on_both_requirement_pages_without_duplicate_frontier_item() {
    let mut state = empty_state();
    state.requirements = vec![
        requirement(
            "req_left",
            "Platform shall prefer the left branch",
            RequirementStatus::Active,
            vec![],
        ),
        requirement(
            "req_right",
            "Platform shall prefer the right branch",
            RequirementStatus::Active,
            vec![],
        ),
    ];
    state.edges = vec![edge(
        EdgeType::Contradicts,
        (NodeType::Requirement, "req_left"),
        (NodeType::Requirement, "req_right"),
    )];
    let resolver = LinkResolver::new(None);
    let corpus = build_corpus(&state, &resolver);

    let left_page = requirement_page(&corpus, "req_left");
    let left_contradictions: Vec<_> = left_page
        .gaps
        .iter()
        .filter(|gap| gap.kind == GapKind::UnresolvedContradictsPair)
        .collect();
    assert_eq!(left_contradictions.len(), 1);
    assert!(left_contradictions[0].detail.contains("req_right"));

    let right_page = requirement_page(&corpus, "req_right");
    let right_contradictions: Vec<_> = right_page
        .gaps
        .iter()
        .filter(|gap| gap.kind == GapKind::UnresolvedContradictsPair)
        .collect();
    assert_eq!(right_contradictions.len(), 1);
    assert!(right_contradictions[0]
        .detail
        .contains("requirement req_right -> requirement req_left"));

    let pair_count = compute_state_gaps(&state)
        .iter()
        .filter(|gap| gap.kind == GapKind::UnresolvedContradictsPair)
        .count();
    assert_eq!(pair_count, 1);
}
