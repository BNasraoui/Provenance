use crate::wiki::model::{PageKind, ScopeIndexPage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, page_shell, title_row};
use super::super::citations::push_gap_citations;
use super::super::fragments::{push_classification_row, push_orphan_group, push_section_open};
use super::super::html::{escape_attr, escape_html};
use super::super::labels::{counted, requirement_status_badge};

/// Renders the scope index page.
pub fn render_index(scope: &str, page: &ScopeIndexPage) -> String {
    let mut main = String::new();
    push_section_open(
        &mut main,
        "sh-requirement",
        Some("i-git-branch"),
        "Root Requirements",
    );
    if page.roots.is_empty() {
        main.push_str("<p class=\"prose\">No requirements recorded in this scope.</p>\n");
    } else {
        main.push_str("<ul class=\"index-list\">\n");
        for entry in &page.roots {
            main.push_str("<li>\n");
            writeln!(
                main,
                "<a class=\"entry-title\" href=\"{}\">{}</a>",
                escape_attr(&entry.link.target.route()),
                escape_html(&entry.link.title)
            )
            .expect("writing to a String should not fail");
            main.push_str(&requirement_status_badge(
                &entry.status,
                entry.resolutions,
                entry.rules,
            ));
            writeln!(
                main,
                "<span class=\"entry-counts\">{} · {} · {}</span>",
                counted(entry.children, "refinement", "refinements"),
                counted(entry.resolutions, "decision", "decisions"),
                counted(entry.rules, "rule", "rules"),
            )
            .expect("writing to a String should not fail");
            main.push_str("</li>\n");
        }
        main.push_str("</ul>\n");
    }
    main.push_str("</section>\n");
    if !page.orphans.is_empty() {
        push_section_open(&mut main, "", None, "Orphaned Records");
        main.push_str("<div class=\"orphan-card\">\n");
        push_orphan_group(&mut main, "Rules nothing produces", &page.orphans.rules);
        push_orphan_group(
            &mut main,
            "Resolutions resolving nothing",
            &page.orphans.resolutions,
        );
        push_orphan_group(
            &mut main,
            "Sources nothing references",
            &page.orphans.sources,
        );
        main.push_str("</div>\n</section>\n");
    }

    let mut margin = String::new();
    if !page.gaps.is_empty() {
        margin.push_str("<h3 class=\"margin-head\">Gaps</h3>\n");
        push_gap_citations(&mut margin, &page.gaps);
    }
    margin.push_str("<div class=\"classification\">\n<h3 class=\"margin-head\">Records</h3>\n");
    push_classification_row(
        &mut margin,
        "i-book-open",
        "Sources",
        &page.counts.sources.to_string(),
        false,
    );
    push_classification_row(
        &mut margin,
        "i-git-branch",
        "Requirements",
        &page.counts.requirements.to_string(),
        false,
    );
    push_classification_row(
        &mut margin,
        "i-scale",
        "Resolutions",
        &page.counts.resolutions.to_string(),
        false,
    );
    push_classification_row(
        &mut margin,
        "i-shield",
        "Rules",
        &page.counts.rules.to_string(),
        false,
    );
    margin.push_str("</div>\n");

    let container = container_html(
        None,
        &title_row(PageKind::ScopeIndex, &page.title, None, &[], &page.scope),
        &main,
        &margin,
    );
    page_shell(scope, "scope-index", &page.title, "", &container, "")
}
