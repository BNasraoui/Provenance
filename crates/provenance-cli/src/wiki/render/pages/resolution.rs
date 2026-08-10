use crate::wiki::model::{PageKind, PageLink, ResolutionPage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::citations::{gap_links, push_gap_citations, push_input_citations};
use super::super::field_notes::field_notes;
use super::super::fragments::{
    push_attribution, push_classification_block, push_classification_link_row,
    push_classification_row, push_prose_section, push_rule_territory_card, push_section_open,
};
use super::super::html::{escape_html, PageLinksRenderer};
use super::super::labels::{format_confidence, resolution_status_word, status_badge};

/// Every titled link this page renders: the requirements it resolves, the
/// requirements it spawned, and the resolution that superseded it.
fn page_links(page: &ResolutionPage) -> Vec<&PageLink> {
    let mut links: Vec<&PageLink> = page.resolves.iter().collect();
    links.extend(&page.spawned);
    links.extend(page.produced_rules.iter().map(|rule| &rule.link));
    links.extend(page.superseded_by.iter());
    links.extend(gap_links(&page.gaps));
    links
}

/// Renders a resolution detail page.
pub fn render_resolution(scope: &str, page: &ResolutionPage) -> String {
    let links = PageLinksRenderer::new(page_links(page));
    let mut main = String::new();
    push_section_open(&mut main, "sh-resolution", Some("i-scale"), "Position");
    writeln!(
        main,
        "<blockquote class=\"position\">{}</blockquote>",
        escape_html(&page.position)
    )
    .expect("writing to a String should not fail");
    main.push_str("</section>\n");
    push_prose_section(
        &mut main,
        "sh-resolution",
        None,
        "Rationale",
        &page.rationale,
    );
    if let Some(context) = &page.context {
        push_prose_section(&mut main, "sh-resolution", None, "Context", context);
    }
    if !page.resolves.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Resolves",
        );
        main.push_str(&links.link_list(&page.resolves));
        main.push_str("</section>\n");
    }
    if !page.spawned.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Spawned Requirements",
        );
        main.push_str(&links.link_list(&page.spawned));
        main.push_str("</section>\n");
    }
    if !page.produced_rules.is_empty() {
        push_section_open(&mut main, "sh-rule", Some("i-shield"), "Produced rules");
        main.push_str("<div class=\"territory\">\n");
        push_rule_territory_card(&mut main, &links, &page.produced_rules);
        main.push_str("</div>\n</section>\n");
    }
    push_attribution(
        &mut main,
        page.made_by.as_deref(),
        page.approved_by.as_deref(),
        page.approved_at,
        resolution_status_word(&page.status),
    );

    let margin = margin_html(page, &links);

    let index = (
        crate::wiki::routes::WikiRoute::Index.path(),
        scope.to_string(),
    );
    let container = container_html(
        Some((PageKind::Resolution, index)),
        &title_row(
            PageKind::Resolution,
            &page.title,
            Some(&status_badge(resolution_status_word(&page.status))),
            &[],
            &page.id.record_id,
        ),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "resolution",
        &page.title,
        &index_breadcrumb(scope),
        &container,
        &field_notes(&page.threads, &page.id),
    )
}

/// The scholarly margin: inputs as citations, then the classification rows.
fn margin_html(page: &ResolutionPage, links: &PageLinksRenderer) -> String {
    let mut margin = String::new();
    if !page.inputs.is_empty() {
        margin.push_str("<h3 class=\"margin-head\">Inputs</h3>\n");
        push_input_citations(&mut margin, &page.inputs);
    }
    if !page.gaps.is_empty() {
        margin.push_str("<h3 class=\"margin-head\">Gaps</h3>\n");
        push_gap_citations(&mut margin, links, &page.gaps);
    }
    let mut rows = String::new();
    if let Some(enforcement) = &page.enforcement {
        push_classification_row(&mut rows, "i-shield", "Enforcement", enforcement, false);
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
    if let Some(review_on) = &page.review_on {
        push_classification_row(&mut rows, "i-calendar", "Review on", review_on, false);
    }
    if let Some(superseded_by) = &page.superseded_by {
        push_classification_link_row(&mut rows, links, "i-scale", "Superseded by", superseded_by);
    }
    push_classification_block(&mut margin, &rows);
    margin
}
