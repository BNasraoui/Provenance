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
pub(super) mod routes;

use crate::wiki::model::WikiCorpus;
use routes::WikiRoute;

pub use pages::{
    render_index, render_not_found, render_requirement, render_resolution, render_rule,
    render_search, render_source, render_topics,
};

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
        WikiRoute::Index,
        &corpus.index.title,
        render_index(scope, &corpus.index),
    )];
    pages.push(rendered(
        WikiRoute::Topics,
        &corpus.topics.title,
        render_topics(scope, &corpus.topics),
    ));
    pages.push(rendered(
        WikiRoute::Search,
        &corpus.search.title,
        render_search(scope, &corpus.search),
    ));
    for page in &corpus.requirements {
        pages.push(rendered(
            WikiRoute::Record(&page.id),
            &page.title,
            render_requirement(scope, page),
        ));
    }
    for page in &corpus.resolutions {
        pages.push(rendered(
            WikiRoute::Record(&page.id),
            &page.title,
            render_resolution(scope, page),
        ));
    }
    for page in &corpus.rules {
        pages.push(rendered(
            WikiRoute::Record(&page.id),
            &page.title,
            render_rule(scope, page),
        ));
    }
    for page in &corpus.sources {
        pages.push(rendered(
            WikiRoute::Record(&page.id),
            &page.title,
            render_source(scope, page),
        ));
    }
    pages
}

fn rendered(route: WikiRoute<'_>, title: &str, html: String) -> RenderedPage {
    RenderedPage {
        route: route.path(),
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
