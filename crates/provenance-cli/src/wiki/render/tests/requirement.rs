use crate::wiki::model::PageKind;

use super::super::render_requirement;
use super::fixtures::{gappy_requirement_fixture, link, requirement_fixture};

#[test]
fn requirement_page_carries_the_mockup_structure() {
    let html = render_requirement("default", &requirement_fixture());
    assert!(html.contains("class=\"accent-bar requirement\""));
    assert!(html.contains("<h1>SaveInvoice shall split each claim item into portions</h1>"));
    assert!(html.contains("type-badge requirement"));
    assert!(html.contains("status-badge discovery"));
    assert!(html.contains("<span class=\"id-chip\">req_saveinvoice_split</span>"));
    assert!(html.contains("Statement"));
    assert!(html.contains("Resolving Decision"));
    assert!(html.contains("blockquote class=\"position\""));
    assert!(html.contains("Downstream Territory"));
    assert!(html.contains("Field Notes"));
}

#[test]
fn requirement_page_links_every_code_reference() {
    let html = render_requirement("default", &requirement_fixture());
    // Rule evidence links to the host blob URL with line anchors.
    assert!(
        html.contains("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L153-L156")
    );
    // Source citation reference pins to the source commit.
    assert!(html.contains("https://github.com/visualcare/vc-api/blob/abc1234/docs/award.md"));
    // Field-note bodies linkify code refs and test-case names.
    assert!(
        html.contains("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L211-L233")
    );
    assert!(html.contains(">testCreateGapInvoiceOnly</a>"));
}

#[test]
fn requirement_page_numbers_source_citations_in_the_margin() {
    let html = render_requirement("default", &requirement_fixture());
    assert!(html.contains("<span class=\"cite-num\">[1]</span>"));
    assert!(html.contains("<a href=\"/sources/source_schads/\">SCHADS Award mapping</a>"));
    assert!(html.contains("clause 10.3"));
}

#[test]
fn requirement_page_shows_lineage_with_the_current_entry_unlinked() {
    let html = render_requirement("default", &requirement_fixture());
    assert!(html.contains("<a href=\"/requirements/req_platform/\">Visualcare platform</a>"));
    assert!(html.contains("<li class=\"current\">SaveInvoice shall split each claim item"));
}

#[test]
fn requirement_page_attributes_borrowed_threads_to_their_parent() {
    let html = render_requirement("default", &requirement_fixture());
    assert!(html.contains("thr_resolution_res_split_0"));
    assert!(html.contains("on resolution res_split"));
    assert!(html.contains("1 message · active"));
    assert!(html.contains(">Agent</span>"));
}

#[test]
fn requirement_page_renders_related_sibling_requirements_after_attribution() {
    let mut page = requirement_fixture();
    page.siblings = vec![
        link(
            PageKind::Requirement,
            "req_budget_split",
            "Budget portions shall reconcile",
        ),
        link(
            PageKind::Requirement,
            "req_zero_suppression",
            "Zero claim items shall be suppressed",
        ),
    ];

    let html = render_requirement("default", &page);
    let attribution_pos = html
        .find("<section class=\"attribution\" aria-label=\"Attribution\">")
        .unwrap();
    let related_pos = html
        .find("<h2 class=\"section-head sh-requirement\"><svg class=\"icon\"><use href=\"#i-git-branch\"/></svg>Related</h2>")
        .expect("sibling requirements should render in a Related section");
    assert!(related_pos > attribution_pos);
    assert!(html.contains(
        "<div class=\"card-head\"><svg class=\"icon\"><use href=\"#i-git-branch\"/></svg>Related Requirements — 2</div>"
    ));
    assert!(html.contains("<ul class=\"link-list\">"));
    assert!(html.contains(
        "<a href=\"/requirements/req_budget_split/\">Budget portions shall reconcile</a>"
    ));
    assert!(html.contains(
        "<a href=\"/requirements/req_zero_suppression/\">Zero claim items shall be suppressed</a>"
    ));
}

#[test]
fn requirement_page_omits_related_section_without_siblings() {
    let html = render_requirement("default", &gappy_requirement_fixture());
    assert!(!html.contains(">Related</h2>"));
    assert!(!html.contains("Related Requirements"));
}

#[test]
fn field_notes_who_shows_a_readable_role_not_the_raw_message_id() {
    let html = render_requirement("default", &requirement_fixture());
    assert!(
        !html.contains("msg_000001"),
        "the internal message id should never be shown as if it were an author name"
    );
    assert!(html.contains("<span class=\"who\">Agent</span>"));
}

#[test]
fn gaps_render_as_dashed_citations_and_are_never_suppressed() {
    let html = render_requirement("default", &gappy_requirement_fixture());
    assert_eq!(html.matches("citation gap").count(), 3);
    assert!(html.contains("source ref points at source_missing"));
    assert!(html.contains("no source refs recorded on this requirement"));
    assert!(html.contains("resolved with no resolving decision"));
}

#[test]
fn gappy_page_keeps_the_fog_visible() {
    let html = render_requirement("default", &gappy_requirement_fixture());
    assert!(html.contains("Which award clauses apply is still unclear."));
}
