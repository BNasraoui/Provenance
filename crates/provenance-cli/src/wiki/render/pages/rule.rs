use crate::wiki::model::{PageKind, PageLink, RulePage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::citations::push_gap_citations;
use super::super::field_notes::field_notes;
use super::super::fragments::{
    push_classification_block, push_classification_row, push_prose_section, push_section_open,
};
use super::super::html::{escape_html, evidence_html, PageLinksRenderer};
use super::super::labels::{rule_status_word, sev_chip, severity_word, status_badge};

/// Every titled link this page renders: the records that produce the rule,
/// the requirements upstream of those, and the sources behind them.
fn page_links(page: &RulePage) -> Vec<&PageLink> {
    let mut links: Vec<&PageLink> = page.produced_by.iter().collect();
    links.extend(&page.requirements);
    links.extend(&page.sources);
    links
}

fn push_rule_function(main: &mut String, page: &RulePage) {
    push_section_open(main, "sh-rule", None, "Rule Function");
    if let Some(binding) = &page.rule_function {
        writeln!(
            main,
            "<p class=\"code-site\"><code>{}</code><span class=\"site-separator\"> — </span>{}</p>",
            escape_html(binding.symbol.as_deref().unwrap_or("Unknown symbol")),
            evidence_html(&binding.location),
        )
        .expect("writing to a String should not fail");
    } else {
        main.push_str("<p class=\"empty-state\">No function bound</p>\n");
    }
    main.push_str("</section>\n");
}

fn push_verifications(main: &mut String, page: &RulePage) {
    push_section_open(main, "sh-rule", Some("i-check-circle"), "Verification");
    if page.verifications.is_empty() {
        main.push_str("<p class=\"empty-state\">Not verified</p>\n");
    } else {
        main.push_str("<ul class=\"verification-list\">\n");
        for site in &page.verifications {
            let outside = if site.outside_defining_module {
                " <span class=\"site-note\">outside defining module</span>"
            } else {
                ""
            };
            writeln!(
                main,
                "<li><span class=\"verification-method\">{}</span> <code>{}</code><span class=\"site-separator\"> — </span>{}{outside}</li>",
                escape_html(&site.method),
                escape_html(site.symbol.as_deref().unwrap_or("Unknown symbol")),
                evidence_html(&site.location),
            )
            .expect("writing to a String should not fail");
        }
        main.push_str("</ul>\n");
    }
    main.push_str("</section>\n");
}

/// Renders a rule detail page.
#[allow(clippy::too_many_lines)]
pub fn render_rule(scope: &str, page: &RulePage) -> String {
    let links = PageLinksRenderer::new(page_links(page));
    let mut main = String::new();
    push_prose_section(&mut main, "sh-rule", None, "Statement", &page.statement);
    if let Some(description) = &page.description {
        push_prose_section(&mut main, "sh-rule", None, "Description", description);
    }
    push_rule_function(&mut main, page);
    push_verifications(&mut main, page);
    if !page.produced_by.is_empty() {
        push_section_open(&mut main, "sh-resolution", Some("i-scale"), "Produced By");
        main.push_str(&links.link_list(&page.produced_by));
        main.push_str("</section>\n");
    }
    if !page.requirements.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Upstream Requirements",
        );
        main.push_str(&links.link_list(&page.requirements));
        main.push_str("</section>\n");
    }
    if !page.sources.is_empty() {
        push_section_open(&mut main, "sh-source", Some("i-book-open"), "Sources");
        main.push_str(&links.link_list(&page.sources));
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
    push_classification_block(&mut margin, &rows);

    let chips = vec![sev_chip(
        severity_word(&page.severity),
        severity_word(&page.severity),
    )];
    let container = container_html(
        Some((
            PageKind::Rule,
            (
                crate::wiki::routes::WikiRoute::Index.path(),
                scope.to_string(),
            ),
        )),
        &title_row(
            PageKind::Rule,
            &page.title,
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
