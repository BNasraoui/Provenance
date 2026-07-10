use crate::wiki::links::InlineRef;

use super::super::html::{escape_attr, escape_html, link_list, linkify_body};
use super::super::labels::{format_confidence, format_date_ms};
use super::fixtures::{colliding_requirement_links, unique_requirement_links};

#[test]
fn escape_html_escapes_markup_characters() {
    assert_eq!(escape_html("a < b & c > d"), "a &lt; b &amp; c &gt; d");
}

#[test]
fn escape_attr_also_escapes_quotes() {
    assert_eq!(
        escape_attr("say \"hi\" & <go>"),
        "say &quot;hi&quot; &amp; &lt;go&gt;"
    );
}

#[test]
fn format_date_ms_renders_epoch_milliseconds_as_a_civil_date() {
    assert_eq!(format_date_ms(1_714_780_800_000), "4 May 2024");
    assert_eq!(format_date_ms(0), "1 Jan 1970");
}

#[test]
fn format_date_ms_handles_dates_before_the_epoch() {
    assert_eq!(format_date_ms(-86_400_000), "31 Dec 1969");
}

#[test]
fn format_confidence_renders_a_percentage() {
    assert_eq!(format_confidence(0.97), "97%");
    assert_eq!(format_confidence(1.0), "100%");
}

#[test]
fn linkify_body_wraps_ref_spans_in_anchors_and_escapes_the_rest() {
    let body = "See src/UseCase.php:153-156 & friends";
    let refs = vec![InlineRef {
        start: 4,
        end: 27,
        label: "src/UseCase.php:153-156".to_string(),
        href: "https://example.test/blob".to_string(),
    }];
    assert_eq!(
        linkify_body(body, &refs),
        "See <a class=\"src\" href=\"https://example.test/blob\">src/UseCase.php:153-156</a> &amp; friends"
    );
}

#[test]
fn linkify_body_escapes_plain_text_when_there_are_no_refs() {
    assert_eq!(linkify_body("a < b", &[]), "a &lt; b");
}

#[test]
fn link_list_disambiguates_identical_titles_with_stable_id_chips() {
    let html = link_list(&colliding_requirement_links());

    assert!(html.contains(
        "Participant budget summary shall pro-rate services</a> <span class=\"id-chip\">…hall_pro</span>"
    ));
    assert!(html.contains(
        "Participant budget summary shall pro-rate services</a> <span class=\"id-chip\">…ll_pro_2</span>"
    ));
}

#[test]
fn link_list_leaves_unique_titles_unchanged() {
    assert_eq!(
        link_list(&unique_requirement_links()),
        "<ul class=\"link-list\">\n\
<li><a href=\"/requirements/req_budget_split/\">Budget portions shall reconcile</a></li>\n\
<li><a href=\"/requirements/req_zero_suppression/\">Zero claim items shall be suppressed</a></li>\n\
</ul>\n"
    );
}

#[test]
fn repeated_links_to_the_same_record_do_not_collide() {
    let link = colliding_requirement_links().remove(0);

    assert!(!link_list(&[link.clone(), link]).contains("class=\"id-chip\""));
}

#[test]
fn link_list_disambiguates_same_id_across_page_kinds() {
    let links = vec![
        super::fixtures::link(
            crate::wiki::model::PageKind::Requirement,
            "shared_id",
            "Shared title",
        ),
        super::fixtures::link(
            crate::wiki::model::PageKind::Resolution,
            "shared_id",
            "Shared title",
        ),
        super::fixtures::link(
            crate::wiki::model::PageKind::Source,
            "shared_id",
            "Shared title",
        ),
    ];

    let html = link_list(&links);
    assert!(html.contains("<span class=\"id-chip\">requirement · shared_id</span>"));
    assert!(html.contains("<span class=\"id-chip\">resolution · shared_id</span>"));
    assert!(html.contains("<span class=\"id-chip\">source · shared_id</span>"));
}

#[test]
fn short_ids_use_the_shortest_suffix_that_distinguishes_records() {
    let links = vec![
        super::fixtures::link(crate::wiki::model::PageKind::Requirement, "req_a1", "Same"),
        super::fixtures::link(crate::wiki::model::PageKind::Requirement, "req_b1", "Same"),
    ];

    let html = link_list(&links);
    assert!(html.contains("<span class=\"id-chip\">req_a1</span>"));
    assert!(html.contains("<span class=\"id-chip\">req_b1</span>"));
}
