use super::fixtures::*;
use crate::wiki::model::GapKind;

#[test]
fn resolution_page_links_requirements_rules_and_spawned_work() {
    let corpus = fixture_corpus();
    let page = resolution_page(&corpus, "res_split");
    assert_eq!(page.resolves.len(), 1);
    assert_eq!(page.resolves[0].target.record_id, "req_child");
    assert_eq!(page.spawned.len(), 1);
    assert_eq!(page.spawned[0].target.record_id, "req_stuck");
    assert_eq!(page.produced_rules.len(), 1);
    assert_eq!(page.produced_rules[0].rule_code, "SAH-INV-001");
    assert!(page.gaps.is_empty());
    assert_eq!(page.threads.len(), 1);
    assert_eq!(page.threads[0].thread_id, "thr_res_split");
}

#[test]
fn resolution_page_flags_orphaned_and_ruleless_decisions() {
    let corpus = fixture_corpus();
    let page = resolution_page(&corpus, "res_orphan");
    assert_eq!(
        gap_kinds(&page.gaps),
        vec![GapKind::OrphanResolution, GapKind::NoProducedRules]
    );
}

#[test]
fn rule_page_traces_back_to_requirements_and_sources() {
    let corpus = fixture_corpus();
    let page = rule_page(&corpus, "rule_001");
    assert_eq!(page.title, "Invoices grouped by participant");
    let produced_by: Vec<&str> = page
        .produced_by
        .iter()
        .map(|link| link.target.record_id.as_str())
        .collect();
    assert_eq!(produced_by, vec!["res_split", "req_child"]);
    assert_eq!(page.requirements.len(), 1);
    assert_eq!(page.requirements[0].target.record_id, "req_child");
    assert_eq!(page.sources.len(), 1);
    assert_eq!(page.sources[0].target.record_id, "source_schads");
    assert_eq!(page.evidence.len(), 1);
    assert_eq!(
        page.evidence[0].href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L59-L69")
    );
    assert!(page.gaps.is_empty());
}

#[test]
fn rule_page_flags_orphan_rules_and_falls_back_to_the_rule_code() {
    let corpus = fixture_corpus();
    let page = rule_page(&corpus, "rule_orphan");
    assert_eq!(page.title, "SAH-INV-999");
    assert_eq!(gap_kinds(&page.gaps), vec![GapKind::OrphanRule]);
    assert!(page.produced_by.is_empty());
}

#[test]
fn source_page_lists_referencing_requirements_and_pins_links() {
    let corpus = fixture_corpus();
    let page = source_page(&corpus, "source_schads");
    assert_eq!(page.referenced_requirements.len(), 1);
    assert_eq!(
        page.referenced_requirements[0].target.record_id,
        "req_child"
    );
    assert_eq!(
        page.reference.as_ref().unwrap().href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/abc1234/docs/award.md")
    );
    assert!(page.gaps.is_empty());
}

#[test]
fn source_page_flags_unreferenced_sources() {
    let corpus = fixture_corpus();
    let page = source_page(&corpus, "source_unused");
    assert_eq!(gap_kinds(&page.gaps), vec![GapKind::UnreferencedSource]);
    assert!(page.referenced_requirements.is_empty());
}
