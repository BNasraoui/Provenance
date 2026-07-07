use super::super::render_corpus;
use super::fixtures::corpus_fixture;

#[test]
fn every_page_is_self_contained_and_theme_aware() {
    let corpus = corpus_fixture();
    for page in render_corpus(&corpus) {
        assert!(page.html.starts_with("<!doctype html>"), "{}", page.route);
        assert!(page.html.contains("data-theme=\"statesman\""));
        assert!(
            page.html
                .contains("<link rel=\"stylesheet\" href=\"/assets/provenance-wiki.css\">"),
            "{}",
            page.route
        );
        for theme in ["statesman", "piano", "latte", "mocha", "dracula"] {
            assert!(
                page.html.contains(&format!("<option value=\"{theme}\"")),
                "{}: missing theme option {theme}",
                page.route
            );
        }
        assert!(page.html.contains("provenance-wiki-theme"));
    }
}

#[test]
fn render_corpus_routes_every_page_under_its_page_id() {
    let corpus = corpus_fixture();
    let routes: Vec<String> = render_corpus(&corpus)
        .into_iter()
        .map(|page| page.route)
        .collect();
    assert_eq!(
        routes,
        vec![
            "/".to_string(),
            "/requirements/req_saveinvoice_split/".to_string(),
            "/requirements/req_stuck/".to_string(),
            "/resolutions/res_split/".to_string(),
            "/rules/rule_sah_inv_016/".to_string(),
            "/sources/source_schads/".to_string(),
        ]
    );
}
