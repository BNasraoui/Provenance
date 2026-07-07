use crate::wiki::model::{PageKind, SourcePage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::citations::push_gap_citations;
use super::super::field_notes::field_notes;
use super::super::fragments::{
    push_classification_block, push_classification_link_row, push_classification_row,
    push_section_open,
};
use super::super::html::{escape_attr, escape_html, evidence_html, link_list};
use super::super::labels::{format_date_ms, source_type_label, status_badge};

/// Renders a source detail page.
pub fn render_source(scope: &str, page: &SourcePage) -> String {
    let mut main = String::new();
    push_section_open(&mut main, "sh-source", Some("i-book-open"), "Reference");
    main.push_str("<ul class=\"link-list\">\n");
    if let Some(url) = &page.url {
        writeln!(
            main,
            "<li><a href=\"{}\">{}</a></li>",
            escape_attr(url),
            escape_html(url)
        )
        .expect("writing to a String should not fail");
    }
    if let Some(reference) = &page.reference {
        writeln!(main, "<li>{}</li>", evidence_html(reference))
            .expect("writing to a String should not fail");
    }
    if let Some(commit_pin) = &page.commit_pin {
        writeln!(
            main,
            "<li>pinned to <code>{}</code></li>",
            escape_html(commit_pin)
        )
        .expect("writing to a String should not fail");
    }
    if page.url.is_none() && page.reference.is_none() && page.commit_pin.is_none() {
        main.push_str("<li>No reference recorded.</li>\n");
    }
    main.push_str("</ul>\n</section>\n");
    if !page.referenced_requirements.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Referenced Requirements",
        );
        main.push_str(&link_list(&page.referenced_requirements));
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
        "i-book-open",
        "Type",
        source_type_label(&page.source_type),
        false,
    );
    if let Some(commit_pin) = &page.commit_pin {
        push_classification_row(&mut rows, "i-git-branch", "Commit pin", commit_pin, true);
    }
    if let Some(effective_date) = page.effective_date {
        push_classification_row(
            &mut rows,
            "i-calendar",
            "Effective",
            &format_date_ms(effective_date),
            false,
        );
    }
    if let Some(review_date) = page.review_date {
        push_classification_row(
            &mut rows,
            "i-calendar",
            "Review",
            &format_date_ms(review_date),
            false,
        );
    }
    if let Some(superseded_by) = &page.superseded_by {
        push_classification_link_row(&mut rows, "i-book-open", "Superseded by", superseded_by);
    }
    push_classification_block(&mut margin, &rows);

    let superseded_badge = page
        .superseded_by
        .as_ref()
        .map(|_| status_badge("superseded"));
    let container = container_html(
        Some((PageKind::Source, ("/".to_string(), scope.to_string()))),
        &title_row(
            PageKind::Source,
            &page.title,
            superseded_badge.as_deref(),
            &[],
            &page.id.record_id,
        ),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "source",
        &page.title,
        &index_breadcrumb(scope),
        &container,
        &field_notes(&page.threads, &page.id),
    )
}
