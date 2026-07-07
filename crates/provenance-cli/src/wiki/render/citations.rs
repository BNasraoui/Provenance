use crate::wiki::model::{GapNotice, InputCitation, SourceCitation};
use std::fmt::Write as _;

use super::html::{escape_html, evidence_html, link_html};
use super::labels::{input_type_label, source_type_label};

pub(in crate::wiki::render) fn push_gap_citations(html: &mut String, gaps: &[GapNotice]) {
    for gap in gaps {
        write!(
            html,
            "<div class=\"citation gap\">\n<div class=\"cite-head\"><span class=\"cite-type\" style=\"color:inherit\">Gap</span></div>\n<p>{}</p>\n</div>\n",
            escape_html(&gap.detail)
        )
        .expect("writing to a String should not fail");
    }
}

pub(in crate::wiki::render) fn push_source_citations(
    html: &mut String,
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
        writeln!(html, "<p>{}{clause}</p>", link_html(&citation.link))
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
