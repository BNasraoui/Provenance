use crate::wiki::links::{EvidenceRef, InlineRef};
use crate::wiki::model::PageLink;
use std::fmt::Write as _;

pub(in crate::wiki::render) fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(in crate::wiki::render) fn escape_attr(text: &str) -> String {
    escape_html(text).replace('"', "&quot;")
}

pub(in crate::wiki::render) fn link_html(link: &PageLink) -> String {
    format!(
        "<a href=\"{}\">{}</a>",
        escape_attr(&link.target.route()),
        escape_html(&link.title)
    )
}

pub(in crate::wiki::render) fn link_list(links: &[PageLink]) -> String {
    let mut html = String::from("<ul class=\"link-list\">\n");
    for link in links {
        writeln!(html, "<li>{}</li>", link_html(link))
            .expect("writing to a String should not fail");
    }
    html.push_str("</ul>\n");
    html
}

pub(in crate::wiki::render) fn evidence_html(evidence: &EvidenceRef) -> String {
    evidence.href.as_ref().map_or_else(
        || escape_html(&evidence.label),
        |href| {
            format!(
                "<a href=\"{}\">{}</a>",
                escape_attr(href),
                escape_html(&evidence.label)
            )
        },
    )
}

pub(in crate::wiki::render) fn icon_svg(symbol: &str) -> String {
    format!("<svg class=\"icon\"><use href=\"#{symbol}\"/></svg>")
}

/// Escapes a field-note body while wrapping each [`InlineRef`] span in an
/// anchor. Spans are byte offsets into `body`, non-overlapping and sorted.
pub(in crate::wiki::render) fn linkify_body(body: &str, refs: &[InlineRef]) -> String {
    let mut html = String::new();
    let mut cursor = 0;
    for inline in refs {
        if inline.start < cursor || inline.end > body.len() {
            continue;
        }
        html.push_str(&escape_html(&body[cursor..inline.start]));
        write!(
            html,
            "<a class=\"src\" href=\"{}\">{}</a>",
            escape_attr(&inline.href),
            escape_html(&inline.label)
        )
        .expect("writing to a String should not fail");
        cursor = inline.end;
    }
    html.push_str(&escape_html(&body[cursor..]));
    html
}
