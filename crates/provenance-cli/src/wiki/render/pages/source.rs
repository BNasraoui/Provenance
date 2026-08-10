use crate::wiki::model::{PageKind, PageLink, SourcePage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::citations::{gap_links, push_gap_citations};
use super::super::field_notes::field_notes;
use super::super::fragments::{
    push_classification_block, push_classification_link_row, push_classification_row,
    push_section_open,
};
use super::super::html::{escape_attr, escape_html, evidence_html, PageLinksRenderer};
use super::super::labels::{format_date_ms, source_type_label, status_badge};

/// Every titled link this page renders: the requirements that reference the
/// source, and the source that superseded it.
fn page_links(page: &SourcePage) -> Vec<&PageLink> {
    let mut links: Vec<&PageLink> = page.referenced_requirements.iter().collect();
    links.extend(page.superseded_by.iter());
    links.extend(gap_links(&page.gaps));
    links
}

/// Renders a source detail page.
pub fn render_source(scope: &str, page: &SourcePage) -> String {
    let links = PageLinksRenderer::new(page_links(page));
    let mut main = String::new();
    push_reference(&mut main, page);
    if !page.referenced_requirements.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Referenced Requirements",
        );
        main.push_str(&links.link_list(&page.referenced_requirements));
        main.push_str("</section>\n");
    }

    let mut margin = String::new();
    if !page.gaps.is_empty() {
        margin.push_str("<h3 class=\"margin-head\">Gaps</h3>\n");
        push_gap_citations(&mut margin, &links, &page.gaps);
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
        push_classification_link_row(
            &mut rows,
            &links,
            "i-book-open",
            "Superseded by",
            superseded_by,
        );
    }
    push_classification_block(&mut margin, &rows);

    let superseded_badge = page
        .superseded_by
        .as_ref()
        .map(|_| status_badge("superseded"));
    let index = (
        crate::wiki::routes::WikiRoute::Index.path(),
        scope.to_string(),
    );
    let container = container_html(
        Some((PageKind::Source, index)),
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

fn push_reference(main: &mut String, page: &SourcePage) {
    push_section_open(main, "sh-source", Some("i-book-open"), "Reference");
    main.push_str("<ul class=\"link-list\">\n");
    if let Some(url) = &page.url {
        if is_web_url(url) {
            writeln!(
                main,
                "<li><a href=\"{}\">{}</a></li>",
                escape_attr(url),
                escape_html(url)
            )
            .expect("writing to a String should not fail");
        } else {
            writeln!(
                main,
                "<li>{} <span class=\"reference-note\">({})</span></li>",
                escape_html(url),
                if has_scheme(url, "file://") {
                    "local file URL is unavailable to wiki readers"
                } else {
                    "URL is unavailable to wiki readers"
                }
            )
            .expect("writing to a String should not fail");
        }
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
}

fn is_web_url(url: &str) -> bool {
    has_scheme(url, "http://") || has_scheme(url, "https://")
}

fn has_scheme(url: &str, scheme: &str) -> bool {
    url.get(..scheme.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme))
}
