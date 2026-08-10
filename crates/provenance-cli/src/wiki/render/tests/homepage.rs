use provenance_macros::verifies;

use super::super::pages::index::{homepage_entry_order_is_valid, homepage_plain_language_is_valid};
use super::super::{pages::render_unfinished, render_corpus};
use super::fixtures::{corpus_fixture, link, unfinished_fixture};
use crate::wiki::model::{DomainGroup, DomainState, HomepageDomain, PageKind};

const RATIFIED_VIEWPORTS: [(u16, u16); 2] = [(1440, 900), (390, 844)];

fn homepage_html() -> String {
    render_corpus(&corpus_fixture())
        .into_iter()
        .find(|page| page.route == "/")
        .expect("the corpus must render a homepage")
        .html
}

#[test]
#[verifies("rule_wiki_homepage_entry_order", examples)]
fn homepage_orders_search_domains_and_traceability_at_ratified_widths() {
    let html = homepage_html();

    for viewport in RATIFIED_VIEWPORTS {
        assert!(
            homepage_entry_order_is_valid(&html, viewport.0, viewport.1),
            "homepage order failed at {viewport:?}"
        );
    }
    assert!(html.contains("data-homepage-domain-row"));
}

#[test]
#[verifies("rule_wiki_homepage_plain_language", examples)]
fn homepage_uses_plain_reader_language() {
    let html = homepage_html();

    assert!(homepage_plain_language_is_valid(&html));
    assert!(!html.to_ascii_lowercase().contains("corpus"));
    assert!(!html.contains(">Atlas<"));
    for everyday_label in ["documentation", "project records", "unfinished"] {
        assert!(html.contains(everyday_label), "missing {everyday_label:?}");
    }
    assert!(html.contains("Browse 1 area"));
    assert!(!html.contains("1 areas"));
}

#[test]
fn homepage_plain_language_check_allows_an_atlas_scope_name() {
    let corpus = corpus_fixture();

    let html = super::super::render_index("Atlas", &corpus.index);

    assert!(
        html.contains("<span class=\"scope\">Atlas</span>"),
        "{html}"
    );
    assert!(homepage_plain_language_is_valid(&html));
}

#[test]
#[verifies("rule_wiki_homepage_search_coverage", examples)]
fn homepage_states_the_search_index_coverage() {
    let html = homepage_html();

    assert!(html.contains("Search covers requirements, decisions, rules, and sources."));
}

#[test]
#[verifies("rule_wiki_homepage_traceability_summary", examples)]
fn homepage_links_one_exact_unfinished_count_without_gap_cards() {
    let html = homepage_html();

    assert_eq!(html.matches("href=\"/unfinished/\"").count(), 1);
    assert_eq!(html.matches("4 unfinished items").count(), 1);
    assert!(!html.contains("citation gap"));
}

#[test]
fn unfinished_route_renders_every_kind_summarized_by_the_homepage() {
    let pages = render_corpus(&corpus_fixture());
    let unfinished = pages
        .iter()
        .find(|page| page.route == "/unfinished/")
        .expect("the complete Unfinished route must be rendered");

    for detail in [
        "Unused API spec",
        "A requirement points to a source that is missing.",
        "A requirement has no source references.",
        "A requirement is marked resolved but has no resolving decision.",
    ] {
        assert!(unfinished.html.contains(detail), "missing {detail:?}");
    }
    assert_eq!(unfinished.html.matches("citation gap").count(), 3);
}

#[test]
fn unfinished_disambiguates_titles_across_all_sections() {
    let mut page = unfinished_fixture();
    page.gaps[0].subject = Some(link(PageKind::Requirement, "req_shared", "Shared title"));
    page.gaps[0].detail = "{subject} points to a source that is missing.".to_string();
    page.orphans.sources[0].link.title = "Shared title".to_string();

    let html = render_unfinished("default", &page);

    assert!(
        html.contains("Shared title <span class=\"id-chip\">…q_shared</span>"),
        "{html}"
    );
    assert!(
        html.contains("Shared title <span class=\"id-chip\">…e_unused</span>"),
        "{html}"
    );
}

#[test]
fn homepage_with_no_authored_domains_shows_the_empty_note_without_a_browse_link() {
    let mut corpus = corpus_fixture();
    corpus.index.domains = Vec::new();
    corpus.index.authored_domain_count = 0;

    let html = render_corpus(&corpus)
        .into_iter()
        .find(|page| page.route == "/")
        .unwrap()
        .html;

    assert!(html.contains("No project areas have been described yet."));
    assert!(!html.contains("data-homepage-domain-row"));
    assert!(!html.contains("browse-all"));
    assert!(!html.contains("Browse all areas"));
    for viewport in RATIFIED_VIEWPORTS {
        assert!(
            homepage_entry_order_is_valid(&html, viewport.0, viewport.1),
            "homepage order failed at {viewport:?}"
        );
    }
}

#[test]
fn homepage_caps_authored_domain_rows() {
    let mut corpus = corpus_fixture();
    corpus.domains.groups = (0..25)
        .map(|number| DomainGroup {
            state: DomainState::Defined {
                id: format!("domain_{number}"),
                name: format!("Area {number}"),
                description: None,
            },
            requirements: Vec::new(),
            rules: Vec::new(),
        })
        .collect();
    corpus.index.domains = (0..25)
        .map(|number| HomepageDomain {
            id: format!("domain_{number}"),
            name: format!("Area {number}"),
            description: None,
            requirements: 0,
            rules: 0,
        })
        .collect();
    corpus.index.authored_domain_count = 25;

    let html = render_corpus(&corpus)
        .into_iter()
        .find(|page| page.route == "/")
        .unwrap()
        .html;

    assert_eq!(html.matches("data-homepage-domain-row").count(), 20);
    assert!(html.contains("Browse all 25 areas"));
}
