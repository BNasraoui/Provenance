use super::super::{render_domains, render_search};
use super::fixtures::{domain_index_fixture, requirement_fixture, search_fixture};

#[test]
fn domain_index_renders_defined_missing_and_unassigned_sections() {
    let html = render_domains("default", &domain_index_fixture());

    assert!(html.contains("id=\"domain-domain_default\""), "{html}");
    assert!(html.contains("id=\"domain-domain_missing\""), "{html}");
    assert!(html.contains("id=\"unassigned-domain-records\""), "{html}");
    assert!(
        html.contains("href=\"/requirements/req_saveinvoice_split/\""),
        "{html}"
    );
    assert!(html.contains("href=\"/rules/rule_sah_inv_016/\""), "{html}");
    assert!(html.contains("Domain record missing"), "{html}");
}

#[test]
fn search_renders_safe_readable_dom_entries_and_shipped_script() {
    let html = render_search("default", &search_fixture());

    assert!(html.contains("id=\"wiki-search\""), "{html}");
    assert!(
        html.contains("data-search-title=\"Invoice &amp; participant\""),
        "{html}"
    );
    assert!(
        html.contains("data-search-statement=\"Invoice &amp; participant statement\""),
        "{html}"
    );
    assert!(
        html.contains("href=\"/requirements/req_saveinvoice_split/\""),
        "{html}"
    );
    assert!(!html.contains("search-index.json"), "{html}");
    assert!(!html.contains("fetch("), "{html}");
    assert!(!html.contains("data-search-entry hidden"), "{html}");
}

#[test]
fn requirement_domain_classification_links_to_its_section() {
    let html = super::super::render_requirement("default", &requirement_fixture());
    assert!(
        html.contains("href=\"/domains/#domain-dom_invoicing\""),
        "{html}"
    );
}
