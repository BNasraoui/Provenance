use crate::wiki::model::{DecisionIndexPage, PageKind};
use crate::wiki::routes::WikiRoute;
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::html::{escape_html, PageLinksRenderer};
use super::super::labels::counted;

pub fn render_decisions(scope: &str, page: &DecisionIndexPage) -> String {
    let mut main = String::new();
    if page.entries.is_empty() {
        main.push_str("<p class=\"empty-note\">No decisions have been recorded.</p>\n");
    } else {
        let links = PageLinksRenderer::new(page.entries.iter().map(|entry| &entry.link));
        main.push_str("<div class=\"decision-index\">\n");
        for entry in &page.entries {
            writeln!(
                main,
                "<article><h2>{}</h2><blockquote class=\"position\">{}</blockquote></article>",
                links.link(&entry.link, None),
                escape_html(&entry.statement),
            )
            .expect("writing to a String should not fail");
        }
        main.push_str("</div>\n");
    }
    let margin = format!(
        "<h3 class=\"margin-head\">Decisions</h3><p class=\"prose\">{}</p>",
        counted(page.entries.len(), "decision", "decisions")
    );
    let container = container_html(
        Some((
            PageKind::ScopeIndex,
            (WikiRoute::Index.path(), scope.to_string()),
        )),
        &title_row(PageKind::DecisionIndex, &page.title, None, &[], &page.scope),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "decision-index",
        &page.title,
        &index_breadcrumb(scope),
        &container,
        "",
    )
}
