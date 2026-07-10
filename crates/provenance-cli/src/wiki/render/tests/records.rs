use crate::wiki::model::{CorpusCounts, OrphanReport, PageId, PageKind, ScopeIndexPage};
use provenance_core::RequirementStatus;

use super::super::{
    render_index, render_not_found, render_requirement, render_resolution, render_rule,
    render_source,
};
use super::fixtures::{
    gappy_requirement_fixture, index_fixture, resolution_fixture, rule_fixture, source_fixture,
};

#[test]
fn resolution_page_renders_inputs_as_citations_and_attribution() {
    let html = render_resolution("default", &resolution_fixture());
    assert!(html.contains("class=\"accent-bar resolution\""));
    assert!(html.contains("status-badge approved"));
    assert!(html.contains("<span class=\"cite-num\">[1]</span>"));
    assert!(html.contains("<span class=\"cite-type\">Technical</span>"));
    assert!(html.contains("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L59-L69"));
    assert!(html.contains("Ben Nasraoui"));
    assert!(html.contains("18 Apr 2026"));
    assert!(html.contains("97%"));
    assert!(html.contains(
        "<a href=\"/requirements/req_saveinvoice_split/\">SaveInvoice shall split each claim item into portions</a>"
    ));
}

#[test]
fn rule_page_links_evidence_but_leaves_prose_references_as_text() {
    let html = render_rule("default", &rule_fixture());
    assert!(html.contains("class=\"accent-bar rule\""));
    assert!(html.contains("SAH-INV-016"));
    assert!(
        html.contains("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L153-L156")
    );
    assert!(html.contains("SCHADS Award clause 10.3"));
    assert!(!html.contains("<a>SCHADS Award clause 10.3</a>"));
    assert!(!html.contains("href=\"\""));
    assert!(html.contains("sev high"));
    assert!(html.contains("prohibition"));
}

#[test]
fn source_page_shows_the_commit_pin_and_referenced_requirements() {
    let html = render_source("default", &source_fixture());
    assert!(html.contains("class=\"accent-bar source\""));
    assert!(html.contains("abc1234"));
    assert!(html.contains("https://example.test/award"));
    assert!(html.contains("4 May 2024"));
    assert!(html.contains(
        "<a href=\"/requirements/req_saveinvoice_split/\">SaveInvoice shall split each claim item into portions</a>"
    ));
}

#[test]
fn index_page_lists_roots_counts_orphans_and_gaps() {
    let html = render_index("default", &index_fixture());
    assert!(html.contains("Provenance atlas — default"));
    assert!(html.contains("<a class=\"entry-title\" href=\"/requirements/req_platform/\">"));
    assert!(html.contains("2 refinements · 1 decision · 1 rule"));
    assert!(html.contains("Orphaned Records"));
    assert!(html.contains("<a href=\"/rules/rule_orphan/\">ORPH-001</a>"));
    assert!(html.contains("<a href=\"/sources/source_unused/\">Unused API spec</a>"));
    assert!(html.contains("citation gap"));
    assert!(html.contains("source_unused is referenced by nothing"));
}

#[test]
fn index_page_on_a_truly_empty_scope_shows_the_honest_empty_state() {
    let page = ScopeIndexPage {
        id: PageId::new(PageKind::ScopeIndex, "default"),
        scope: "default".to_string(),
        title: "Provenance atlas — default".to_string(),
        counts: CorpusCounts::default(),
        roots: vec![],
        gaps: vec![],
        orphans: OrphanReport {
            rules: vec![],
            resolutions: vec![],
            sources: vec![],
        },
    };
    let html = render_index("default", &page);
    assert!(html.contains("No requirements recorded in this scope."));
    assert!(!html.contains("Orphaned Records"));
    assert!(!html.contains("class=\"margin-head\">Gaps"));
    assert!(!html.contains("citation gap"));
}

#[test]
fn index_marks_resolved_requirements_without_decisions_or_rules_unbacked() {
    let mut page = index_fixture();
    page.roots[0].status = RequirementStatus::Resolved;
    page.roots[0].resolutions = 0;
    page.roots[0].rules = 0;

    let html = render_index("default", &page);
    assert!(html.contains("status-badge resolved-unbacked"));
    assert!(html.contains("Resolved (unbacked)"));
}

#[test]
fn not_found_page_names_the_missing_path() {
    let html = render_not_found("default", "/rules/missing/");
    assert!(html.contains("Page not found"));
    assert!(html.contains("/rules/missing/"));
}

#[test]
fn rendered_text_is_html_escaped() {
    let mut page = gappy_requirement_fixture();
    page.title = "Overtime > 38h & \"loading\" <rules>".to_string();
    let html = render_requirement("default", &page);
    assert!(
        html.contains("Overtime &gt; 38h &amp; &quot;loading&quot; &lt;rules&gt;")
            || html.contains("Overtime &gt; 38h &amp; \"loading\" &lt;rules&gt;")
    );
    assert!(!html.contains("<rules>"));
}
