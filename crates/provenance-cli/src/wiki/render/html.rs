use crate::wiki::links::{EvidenceRef, InlineRef};
use crate::wiki::model::{PageId, PageLink};
use std::collections::HashMap;
use std::fmt::Write as _;

use super::labels::kind_label;

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

pub(in crate::wiki::render) struct PageLinksRenderer {
    targets_by_title: HashMap<String, Vec<PageId>>,
}

impl PageLinksRenderer {
    pub(in crate::wiki::render) fn new<'a>(links: impl IntoIterator<Item = &'a PageLink>) -> Self {
        let mut targets_by_title: HashMap<String, Vec<PageId>> = HashMap::new();
        for link in links {
            let targets = targets_by_title.entry(link.title.clone()).or_default();
            if !targets.contains(&link.target) {
                targets.push(link.target.clone());
            }
        }
        targets_by_title.retain(|_, targets| targets.len() > 1);
        Self { targets_by_title }
    }

    pub(in crate::wiki::render) fn link(&self, link: &PageLink, class: Option<&str>) -> String {
        let class = class.map_or_else(String::new, |class| {
            format!(" class=\"{}\"", escape_attr(class))
        });
        format!(
            "<a{class} href=\"{}\">{}{}</a>",
            escape_attr(&link.target.route()),
            escape_html(&link.title),
            self.collision_chip(link)
        )
    }

    pub(in crate::wiki::render) fn text(&self, link: &PageLink) -> String {
        format!("{}{}", escape_html(&link.title), self.collision_chip(link))
    }

    fn collision_chip(&self, link: &PageLink) -> String {
        let mut html = String::new();
        if let Some(targets) = self.targets_by_title.get(&link.title) {
            let suffix = shortest_distinct_suffix(&link.target, targets);
            let kind = targets.iter().any(|other| {
                other.kind != link.target.kind && other.record_id == link.target.record_id
            });
            write!(
                html,
                " <span class=\"id-chip\">{}{}{}{}</span>",
                if kind {
                    kind_label(link.target.kind)
                } else {
                    ""
                },
                if kind { " · " } else { "" },
                if suffix.len() < link.target.record_id.len() {
                    "…"
                } else {
                    ""
                },
                escape_html(suffix)
            )
            .expect("writing to a String should not fail");
        }
        html
    }
}

pub(in crate::wiki::render) fn link_list(links: &[PageLink]) -> String {
    let mut html = String::from("<ul class=\"link-list\">\n");
    let renderer = PageLinksRenderer::new(links);
    for link in links {
        writeln!(html, "<li>{}</li>", renderer.link(link, None))
            .expect("writing to a String should not fail");
    }
    html.push_str("</ul>\n");
    html
}

fn shortest_distinct_suffix<'a>(target: &'a PageId, colliding_targets: &[PageId]) -> &'a str {
    let id = &target.record_id;
    let boundaries: Vec<usize> = id.char_indices().map(|(index, _)| index).collect();
    if boundaries.is_empty() {
        return id;
    }
    let minimum_length = 8.min(boundaries.len());
    for length in minimum_length..=boundaries.len() {
        let suffix = &id[boundaries[boundaries.len() - length]..];
        if colliding_targets
            .iter()
            .all(|other| other == target || !other.record_id.ends_with(suffix))
        {
            return suffix;
        }
    }
    id
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
