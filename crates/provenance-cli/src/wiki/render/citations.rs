use crate::wiki::model::{GapNotice, InputCitation, SourceCitation};
use std::fmt::Write as _;

use super::html::{escape_html, evidence_html, PageLinksRenderer};
use super::labels::{input_type_label, source_type_label};

pub(in crate::wiki::render) fn gap_links(gaps: &[GapNotice]) -> Vec<&crate::wiki::model::PageLink> {
    gaps.iter()
        .flat_map(|gap| gap.subject.iter().chain(gap.related.iter()))
        .collect()
}

pub(in crate::wiki::render) fn gap_detail_html(
    links: &PageLinksRenderer,
    gap: &GapNotice,
) -> String {
    let mut html = String::new();
    let mut remaining = gap.detail.as_str();
    while let Some((index, token, link)) = next_gap_token(remaining, gap) {
        html.push_str(&escape_html(&remaining[..index]));
        html.push_str(&links.link(link, None));
        remaining = &remaining[index + token.len()..];
    }
    html.push_str(&escape_html(remaining));
    html
}

fn next_gap_token<'a>(
    detail: &'a str,
    gap: &'a GapNotice,
) -> Option<(usize, &'static str, &'a crate::wiki::model::PageLink)> {
    let subject = gap.subject.as_ref().and_then(|link| {
        detail
            .find("{subject}")
            .map(|index| (index, "{subject}", link))
    });
    let related = gap.related.as_ref().and_then(|link| {
        detail
            .find("{related}")
            .map(|index| (index, "{related}", link))
    });
    match (subject, related) {
        (Some(subject), Some(related)) => Some(if subject.0 <= related.0 {
            subject
        } else {
            related
        }),
        (Some(subject), None) => Some(subject),
        (None, Some(related)) => Some(related),
        (None, None) => None,
    }
}

pub(in crate::wiki::render) fn push_gap_citations(
    html: &mut String,
    links: &PageLinksRenderer,
    gaps: &[GapNotice],
) {
    for gap in gaps {
        write!(
            html,
            "<div class=\"citation gap\">\n<div class=\"cite-head\"><span class=\"cite-type\" style=\"color:inherit\">Gap</span></div>\n<p>{}</p>\n</div>\n",
            gap_detail_html(links, gap)
        )
        .expect("writing to a String should not fail");
    }
}

pub(in crate::wiki::render) fn push_source_citations(
    html: &mut String,
    links: &PageLinksRenderer,
    sources: &[SourceCitation],
) {
    for (index, citation) in sources.iter().enumerate() {
        write!(
            html,
            "<div class=\"citation\">\n<div class=\"cite-head\"><span class=\"cite-num\">[{}]</span><span class=\"cite-type\">{}</span></div>\n",
            index + 1,
            source_type_label(&citation.source_type)
        )
        .expect("writing to a String should not fail");
        let clause = citation
            .clause
            .as_ref()
            .map(|clause| format!(" — {}", escape_html(clause)))
            .unwrap_or_default();
        writeln!(html, "<p>{}{clause}</p>", links.link(&citation.link, None))
            .expect("writing to a String should not fail");
        if let Some(reference) = &citation.reference {
            writeln!(
                html,
                "<p class=\"cite-ref\">{}</p>",
                evidence_html(reference)
            )
            .expect("writing to a String should not fail");
        }
        html.push_str("</div>\n");
    }
}

pub(in crate::wiki::render) fn push_input_citations(html: &mut String, inputs: &[InputCitation]) {
    for (index, citation) in inputs.iter().enumerate() {
        write!(
            html,
            "<div class=\"citation\">\n<div class=\"cite-head\"><span class=\"cite-num\">[{}]</span><span class=\"cite-type\">{}</span></div>\n<p>{}</p>\n<p class=\"cite-ref\">{}</p>\n</div>\n",
            index + 1,
            input_type_label(&citation.input_type),
            escape_html(&citation.summary),
            evidence_html(&citation.reference)
        )
        .expect("writing to a String should not fail");
    }
}
