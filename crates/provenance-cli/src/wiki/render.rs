//! Renders the wiki page model to semantic HTML.
//!
//! Pages follow the design mockup: type-colored accent bar, top chrome with
//! wordmark and breadcrumb, display-serif title with type and status badges,
//! a 780px body next to a 220px scholarly margin, and a full-width Field
//! Notes band. Everything is a pure `model -> String` function; the CSS and
//! theme switcher are vendored in [`crate::wiki::theme`].

mod chrome;
mod citations;
mod field_notes;
mod fragments;
mod html;
mod labels;
mod pages;

use crate::wiki::model::{PageId, WikiCorpus};

pub use pages::{
    render_index, render_not_found, render_requirement, render_resolution, render_rule,
    render_search, render_source, render_topics,
};

/// Route the vendored stylesheet is referenced under; the server (or static
/// build) must expose [`crate::wiki::theme::WIKI_CSS`] here.
pub const WIKI_CSS_ROUTE: &str = "/assets/provenance-wiki.css";
/// Machine-readable copy of the requirement/rule search corpus.
pub const SEARCH_INDEX_ROUTE: &str = "/assets/search-index.json";

/// One rendered page: its canonical route, title, and full HTML document.
#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub route: String,
    pub title: String,
    pub html: String,
}

/// Renders every page in the corpus, index first, then requirements,
/// resolutions, rules, and sources in record order.
pub fn render_corpus(corpus: &WikiCorpus) -> Vec<RenderedPage> {
    let scope = corpus.scope.as_str();
    let mut pages = vec![rendered(
        &corpus.index.id,
        &corpus.index.title,
        render_index(scope, &corpus.index),
    )];
    pages.push(rendered(
        &corpus.topics.id,
        &corpus.topics.title,
        render_topics(scope, &corpus.topics),
    ));
    pages.push(rendered(
        &corpus.search.id,
        &corpus.search.title,
        render_search(scope, &corpus.search),
    ));
    for page in &corpus.requirements {
        pages.push(rendered(
            &page.id,
            &page.title,
            render_requirement(scope, page),
        ));
    }
    for page in &corpus.resolutions {
        pages.push(rendered(
            &page.id,
            &page.title,
            render_resolution(scope, page),
        ));
    }
    for page in &corpus.rules {
        pages.push(rendered(&page.id, &page.title, render_rule(scope, page)));
    }
    for page in &corpus.sources {
        pages.push(rendered(&page.id, &page.title, render_source(scope, page)));
    }
    pages
}

fn rendered(id: &PageId, title: &str, html: String) -> RenderedPage {
    RenderedPage {
        route: id.route(),
        title: title.to_string(),
        html,
    }
}

#[cfg(test)]
mod tests {
    #[path = "corpus.rs"]
    mod corpus;
    #[path = "discovery.rs"]
    mod discovery;
    #[path = "fixtures.rs"]
    mod fixtures;
    #[path = "formatting.rs"]
    mod formatting;
    #[path = "records.rs"]
    mod records;
    #[path = "requirement.rs"]
    mod requirement;

    use fixtures::{gappy_requirement_fixture, index_fixture, requirement_fixture};

    use super::{render_index, render_requirement};

    #[test]
    fn snapshot_requirement_page_with_rules_and_thread() {
        insta::assert_snapshot!(render_requirement("default", &requirement_fixture()));
    }

    #[test]
    fn snapshot_requirement_page_with_gaps() {
        insta::assert_snapshot!(render_requirement("default", &gappy_requirement_fixture()));
    }

    #[test]
    fn snapshot_scope_index_page() {
        insta::assert_snapshot!(render_index("default", &index_fixture()));
    }
}
