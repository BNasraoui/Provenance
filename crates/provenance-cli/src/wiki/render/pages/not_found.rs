use super::super::chrome::{index_breadcrumb, page_shell};
use super::super::html::escape_html;
use super::super::routes::WikiRoute;

/// Renders the not-found page served for unknown routes.
pub fn render_not_found(scope: &str, path: &str) -> String {
    let container = format!(
        "<div class=\"title-row\">\n<div>\n<h1>Page not found</h1>\n</div>\n</div>\n\
         <div class=\"body-grid\">\n<div class=\"body-main\">\n\
         <p class=\"prose\">No wiki page is served at <code>{}</code>.</p>\n\
         <p class=\"prose\"><a href=\"{}\">Back to the scope index.</a></p>\n\
         </div>\n</div>\n",
        escape_html(path),
        WikiRoute::Index.path()
    );
    page_shell(
        scope,
        "scope-index",
        "Page not found",
        &index_breadcrumb(scope),
        &container,
        "",
    )
}
