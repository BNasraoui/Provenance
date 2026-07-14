use crate::wiki::model::{PageKind, SearchIndexPage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, search_page_shell, title_row};
use super::super::html::{escape_attr, escape_html, PageLinksRenderer};
use super::super::labels::{kind_class, kind_label};
use super::super::routes::WikiRoute;

pub fn render_search(scope: &str, page: &SearchIndexPage) -> String {
    let mut main = String::new();
    write!(
        main,
        "<form class=\"search-box\" action=\"{}\" method=\"get\">\n\
         <label for=\"wiki-search\">Search requirement and rule text</label>\n\
         <div><input id=\"wiki-search\" name=\"q\" type=\"search\" autocomplete=\"off\" \
         placeholder=\"e.g. invoice participant\"><button type=\"submit\">Search</button></div>\n\
         </form>\n",
        WikiRoute::Search.path()
    )
    .expect("writing to a String should not fail");
    main.push_str(
        "<p id=\"search-summary\" class=\"search-summary\" aria-live=\"polite\">Type one or more words to search titles and statements.</p>\n\
         <noscript><p class=\"data-note\">Search filtering requires the vendored JavaScript; the index itself remains in this page.</p></noscript>\n",
    );
    if page.entries.is_empty() {
        main.push_str(
            "<p class=\"empty-note\">No requirements or rules are available to search.</p>\n",
        );
    }
    main.push_str("<ol id=\"search-results\" class=\"search-results\">\n");
    let links = PageLinksRenderer::new(page.entries.iter().map(|entry| &entry.link));
    for entry in &page.entries {
        let kind = entry.link.target.kind.into();
        writeln!(
            main,
            "<li data-search-entry data-search-title=\"{}\" data-search-statement=\"{}\">\n\
                 <span class=\"type-badge {}\">{}</span>\n\
                 {}<p>{}</p></li>",
            escape_attr(&entry.link.title),
            escape_attr(&entry.statement),
            kind_class(kind),
            kind_label(kind),
            links.link(&entry.link, None),
            escape_html(&entry.statement),
        )
        .expect("writing to a String should not fail");
    }
    main.push_str("</ol>\n");
    let margin = format!(
        "<h3 class=\"margin-head\">Indexed fields</h3><p class=\"prose\">Titles and statements from {} requirements and rules. All words must match; title matches rank first.</p>",
        page.entries.len()
    );
    let container = container_html(
        Some((
            PageKind::ScopeIndex,
            (WikiRoute::Index.path(), scope.to_string()),
        )),
        &title_row(PageKind::SearchIndex, &page.title, None, &[], &page.scope),
        &main,
        &margin,
    );
    search_page_shell(
        scope,
        "search-index",
        &page.title,
        &index_breadcrumb(scope),
        &container,
    )
}
