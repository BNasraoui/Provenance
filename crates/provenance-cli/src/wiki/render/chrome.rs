use super::WIKI_CSS_ROUTE;
use crate::wiki::model::{LineageEntry, PageKind};
use crate::wiki::theme::{ICON_DEFS, THEME_SCRIPT};
use std::fmt::Write as _;

use super::html::{escape_attr, escape_html, icon_svg};
use super::labels::{kind_class, kind_icon, kind_label};

pub(in crate::wiki::render) fn page_shell(
    scope: &str,
    kind_class: &str,
    title: &str,
    breadcrumb: &str,
    container: &str,
    field_notes: &str,
) -> String {
    let mut html = String::new();
    html.push_str("<!doctype html>\n<html lang=\"en\" data-theme=\"statesman\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    writeln!(
        html,
        "<title>{} - Provenance Wiki</title>",
        escape_html(title)
    )
    .expect("writing to a String should not fail");
    writeln!(html, "<link rel=\"stylesheet\" href=\"{WIKI_CSS_ROUTE}\">")
        .expect("writing to a String should not fail");
    html.push_str("</head>\n<body>\n");
    html.push_str(ICON_DEFS.trim_start_matches('\n'));
    html.push_str("<header class=\"chrome\">\n<div class=\"chrome-inner\">\n");
    writeln!(
        html,
        "<span class=\"wordmark\">Provenance<span class=\"scope\">{}</span></span>",
        escape_html(scope)
    )
    .expect("writing to a String should not fail");
    writeln!(html, "<nav aria-label=\"Breadcrumb\">{breadcrumb}</nav>")
        .expect("writing to a String should not fail");
    html.push_str(
        "<label class=\"theme-select\">Theme\n<select id=\"theme-select\">\n\
         <option value=\"statesman\" selected>Statesman</option>\n\
         <option value=\"piano\">Piano</option>\n\
         <option value=\"latte\">Catppuccin Latte</option>\n\
         <option value=\"mocha\">Catppuccin Mocha</option>\n\
         <option value=\"dracula\">Dracula</option>\n\
         </select>\n</label>\n",
    );
    html.push_str("</div>\n</header>\n");
    writeln!(
        html,
        "<div class=\"accent-bar {kind_class}\" aria-hidden=\"true\"></div>"
    )
    .expect("writing to a String should not fail");
    write!(html, "<div class=\"container\">\n{container}</div>\n")
        .expect("writing to a String should not fail");
    html.push_str(field_notes);
    html.push_str("<script>");
    html.push_str(THEME_SCRIPT);
    html.push_str("</script>\n</body>\n</html>\n");
    html
}

pub(in crate::wiki::render) fn container_html(
    back: Option<(PageKind, (String, String))>,
    title_row: &str,
    main: &str,
    margin: &str,
) -> String {
    let mut html = String::new();
    if let Some((kind, (route, title))) = back {
        write!(
            html,
            "<a class=\"back-link {}\" href=\"{}\">\n{}{}\n</a>\n",
            kind_class(kind),
            escape_attr(&route),
            icon_svg("i-arrow-left"),
            escape_html(&title)
        )
        .expect("writing to a String should not fail");
    }
    html.push_str(title_row);
    write!(
        html,
        "<div class=\"body-grid\">\n<div class=\"body-main\">\n{main}</div>\n\
         <aside class=\"body-margin\">\n{margin}</aside>\n</div>\n"
    )
    .expect("writing to a String should not fail");
    html
}

pub(in crate::wiki::render) fn title_row(
    kind: PageKind,
    title: &str,
    status_badge: Option<&str>,
    chips: &[String],
    id_chip: &str,
) -> String {
    let mut html = String::new();
    write!(
        html,
        "<div class=\"title-row\">\n<svg class=\"icon {}\" aria-hidden=\"true\"><use href=\"#{}\"/></svg>\n<div>\n<h1>{}</h1>\n<div class=\"badge-row\">\n",
        kind_class(kind),
        kind_icon(kind),
        escape_html(title)
    )
    .expect("writing to a String should not fail");
    if kind != PageKind::ScopeIndex {
        writeln!(
            html,
            "<span class=\"type-badge {}\">{}{}</span>",
            kind_class(kind),
            icon_svg(kind_icon(kind)),
            kind_label(kind)
        )
        .expect("writing to a String should not fail");
    }
    if let Some(badge) = status_badge {
        html.push_str(badge);
        html.push('\n');
    }
    for chip in chips {
        html.push_str(chip);
        html.push('\n');
    }
    writeln!(
        html,
        "<span class=\"id-chip\">{}</span>",
        escape_html(id_chip)
    )
    .expect("writing to a String should not fail");
    html.push_str("</div>\n</div>\n</div>\n");
    html
}

pub(in crate::wiki::render) fn breadcrumb_from_lineage(lineage: &[LineageEntry]) -> String {
    let renderer = super::html::PageLinksRenderer::new(lineage.iter().map(|entry| &entry.link));
    let ancestors: Vec<String> = lineage
        .iter()
        .filter(|entry| !entry.is_current)
        .map(|entry| renderer.link(&entry.link, None))
        .collect();
    ancestors.join(" <span class=\"sep\">›</span> ")
}

pub(in crate::wiki::render) fn index_breadcrumb(scope: &str) -> String {
    format!("<a href=\"/\">{}</a>", escape_html(scope))
}
