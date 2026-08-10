use crate::wiki::model::PageKind;

use super::super::{
    render_not_found, render_requirement, render_resolution, render_rule, render_source,
};
use super::fixtures::{
    gappy_requirement_fixture, resolution_fixture, rule_fixture, source_fixture,
};

#[test]
fn resolution_page_renders_inputs_as_citations_and_attribution() {
    let html = render_resolution("default", &resolution_fixture());
    assert!(html.contains("class=\"accent-bar resolution\""));
    assert!(html.contains("status-badge approved"));
    assert!(html.contains("<span class=\"cite-num\">[1]</span>"));
    assert!(html.contains("<span class=\"cite-type\">Technical</span>"));
    assert!(html.contains("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L59-L69"));
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
    assert!(html.contains("Suppress line emission for fully zero claim items"));
    assert!(
        html.contains("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156")
    );
    assert!(html.contains("SCHADS Award clause 10.3"));
    assert!(!html.contains("<a>SCHADS Award clause 10.3</a>"));
    assert!(!html.contains("href=\"\""));
    assert!(html.contains("sev high"));
}

#[test]
fn rule_page_disambiguates_mixed_kind_producers_with_the_same_id() {
    let mut page = rule_fixture();
    page.produced_by = vec![
        super::fixtures::link(PageKind::Resolution, "shared_id", "Shared producer"),
        super::fixtures::link(PageKind::Requirement, "shared_id", "Shared producer"),
    ];

    let html = render_rule("default", &page);
    assert!(html.contains("Produced By"));
    assert!(html.contains("<span class=\"id-chip\">Resolution · shared_id</span>"));
    assert!(html.contains("<span class=\"id-chip\">Requirement · shared_id</span>"));
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
