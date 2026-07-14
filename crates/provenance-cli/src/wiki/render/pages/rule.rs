use crate::wiki::model::{PageKind, RulePage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::citations::push_gap_citations;
use super::super::field_notes::field_notes;
use super::super::fragments::{
    push_classification_block, push_classification_row, push_prose_section, push_section_open,
};
use super::super::html::{evidence_html, link_list};
use super::super::labels::{
    format_confidence, modality_word, rule_status_word, rule_type_word, sev_chip, severity_word,
    status_badge,
};
use super::super::routes::WikiRoute;

/// Renders a rule detail page.
#[allow(clippy::too_many_lines)]
pub fn render_rule(scope: &str, page: &RulePage) -> String {
    let mut main = String::new();
    push_prose_section(&mut main, "sh-rule", None, "Statement", &page.statement);
    if let Some(description) = &page.description {
        push_prose_section(&mut main, "sh-rule", None, "Description", description);
    }
    if !page.evidence.is_empty() {
        push_section_open(&mut main, "sh-rule", Some("i-book-open"), "Evidence");
        main.push_str("<ul class=\"evidence-list\">\n");
        for evidence in &page.evidence {
            writeln!(main, "<li>{}</li>", evidence_html(evidence))
                .expect("writing to a String should not fail");
        }
        main.push_str("</ul>\n</section>\n");
    }
    if !page.produced_by.is_empty() {
        push_section_open(&mut main, "sh-resolution", Some("i-scale"), "Produced By");
        main.push_str(&link_list(&page.produced_by));
        main.push_str("</section>\n");
    }
    if !page.requirements.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Upstream Requirements",
        );
        main.push_str(&link_list(&page.requirements));
        main.push_str("</section>\n");
    }
    if !page.sources.is_empty() {
        push_section_open(&mut main, "sh-source", Some("i-book-open"), "Sources");
        main.push_str(&link_list(&page.sources));
        main.push_str("</section>\n");
    }

    let mut margin = String::new();
    if !page.gaps.is_empty() {
        margin.push_str("<h3 class=\"margin-head\">Gaps</h3>\n");
        push_gap_citations(&mut margin, &page.gaps);
    }
    let mut rows = String::new();
    push_classification_row(
        &mut rows,
        "i-gauge",
        "Severity",
        severity_word(&page.severity),
        false,
    );
    if let Some(modality) = &page.modality {
        push_classification_row(
            &mut rows,
            "i-shield",
            "Modality",
            modality_word(modality),
            false,
        );
    }
    if let Some(rule_type) = &page.rule_type {
        push_classification_row(
            &mut rows,
            "i-book-open",
            "Type",
            rule_type_word(rule_type),
            false,
        );
    }
    if let Some(confidence) = page.confidence {
        push_classification_row(
            &mut rows,
            "i-gauge",
            "Confidence",
            &format_confidence(confidence),
            false,
        );
    }
    if let Some(extraction_method) = &page.extraction_method {
        push_classification_row(
            &mut rows,
            "i-search",
            "Extraction",
            extraction_method,
            false,
        );
    }
    if let Some(source_document) = &page.source_document {
        push_classification_row(&mut rows, "i-book-open", "Document", source_document, true);
    }
    if let Some(source_section) = &page.source_section {
        push_classification_row(&mut rows, "i-book-open", "Section", source_section, true);
    }
    push_classification_block(&mut margin, &rows);

    let severity_chip = sev_chip(severity_word(&page.severity), severity_word(&page.severity));
    let modality_chip = page
        .modality
        .as_ref()
        .map(|modality| sev_chip("modality", modality_word(modality)));
    let mut chips = vec![severity_chip];
    chips.extend(modality_chip);
    let container = container_html(
        Some((PageKind::Rule, (WikiRoute::Index.path(), scope.to_string()))),
        &title_row(
            PageKind::Rule,
            &format!("{} — {}", page.rule_code, page.title),
            Some(&status_badge(rule_status_word(&page.status))),
            &chips,
            &page.id.record_id,
        ),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "rule",
        &page.title,
        &index_breadcrumb(scope),
        &container,
        &field_notes(&page.threads, &page.id),
    )
}
