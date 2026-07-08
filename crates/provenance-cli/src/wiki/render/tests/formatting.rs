use crate::wiki::links::InlineRef;

use super::super::html::{escape_attr, escape_html, linkify_body};
use super::super::labels::{format_confidence, format_date_ms};

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
