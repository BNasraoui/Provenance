use crate::wiki::model::{FindingsPage, PageKind};
use crate::wiki::routes::WikiRoute;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::citations::push_gap_citations;
use super::super::labels::counted;

pub fn render_findings(scope: &str, page: &FindingsPage) -> String {
    let mut main = String::new();
    if page.findings.is_empty() {
        main.push_str("<p class=\"empty-note\">No missing evidence was found.</p>\n");
    } else {
        main.push_str(
            "<p class=\"prose\">Every current traceability finding is listed below.</p>\n",
        );
        push_gap_citations(&mut main, &page.findings);
    }
    let margin = format!(
        "<h3 class=\"margin-head\">Findings</h3><p class=\"prose\">{}</p>",
        counted(page.findings.len(), "finding", "findings")
    );
    let container = container_html(
        Some((
            PageKind::ScopeIndex,
            (WikiRoute::Index.path(), scope.to_string()),
        )),
        &title_row(PageKind::Findings, &page.title, None, &[], &page.scope),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "findings",
        &page.title,
        &index_breadcrumb(scope),
        &container,
        "",
    )
}
