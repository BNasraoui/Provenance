use crate::wiki::model::{DomainIndexPage, DomainState, PageKind, PageLink, SearchEntry};
use crate::wiki::routes::{domain_anchor, WikiRoute, UNASSIGNED_DOMAIN_ANCHOR};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::html::{escape_attr, escape_html, icon_svg, PageLinksRenderer};
use super::super::labels::counted;

/// Every titled link this page renders: the requirements and rules of every
/// domain group, gathered so one group's title collides visibly with another's.
fn page_links(page: &DomainIndexPage) -> Vec<&PageLink> {
    let mut links: Vec<&PageLink> = Vec::new();
    for group in &page.groups {
        links.extend(&group.requirements);
        links.extend(&group.rules);
    }
    links.extend(page.all_requirements.iter().map(|entry| &entry.link));
    links.extend(page.all_rules.iter().map(|entry| &entry.link));
    links
}

pub fn render_domains(scope: &str, page: &DomainIndexPage) -> String {
    let links = PageLinksRenderer::new(page_links(page));
    let mut main = String::new();
    if page.authored_group_count == 0 {
        main.push_str("<p class=\"empty-note\">No domains have been authored in this scope. All requirements and rules are listed below.</p>\n");
        push_all_records(
            &mut main,
            &links,
            "All requirements",
            &page.all_requirements,
            false,
        );
        push_all_records(&mut main, &links, "All rules", &page.all_rules, true);
    }
    for group in page.groups.iter().filter(|_| page.authored_group_count > 0) {
        let (anchor, name, domain_id, description, gap) = match &group.state {
            DomainState::Defined {
                id,
                name,
                description,
            } => (
                domain_anchor(id),
                name.as_str(),
                Some(id.as_str()),
                description.as_deref(),
                false,
            ),
            DomainState::Missing { id } => (
                domain_anchor(id),
                "Missing domain",
                Some(id.as_str()),
                Some("Domain record missing; membership follows the recorded domain ID."),
                true,
            ),
            DomainState::Unassigned => (
                UNASSIGNED_DOMAIN_ANCHOR.to_string(),
                "Unassigned",
                None,
                Some("Requirements without a Domain and rules without Domain-backed provenance."),
                true,
            ),
        };
        writeln!(
            main,
            "<section class=\"domain-group{}\" id=\"{}\">\n<h2>{}</h2>",
            if gap { " domain-gap" } else { "" },
            escape_attr(&anchor),
            escape_html(name)
        )
        .expect("writing to a String should not fail");
        if let Some(domain_id) = domain_id {
            writeln!(
                main,
                "<code class=\"domain-id\">{}</code>",
                escape_html(domain_id)
            )
            .expect("writing to a String should not fail");
        }
        if let Some(description) = description {
            writeln!(main, "<p class=\"prose\">{}</p>", escape_html(description))
                .expect("writing to a String should not fail");
        }
        if gap {
            main.push_str(
                "<p class=\"data-note\">This group surfaces incomplete taxonomy data without dropping reader-visible records.</p>\n",
            );
        }
        if group.requirements.is_empty() && group.rules.is_empty() {
            main.push_str(
                "<p class=\"empty-note\">No requirements or rules are assigned to this domain.</p>\n",
            );
        } else {
            push_group(
                &mut main,
                &links,
                "Requirements",
                "requirement",
                &group.requirements,
            );
            push_group(&mut main, &links, "Rules", "rule", &group.rules);
        }
        main.push_str("</section>\n");
    }
    let margin = domain_margin(page);
    let container = container_html(
        Some((
            PageKind::ScopeIndex,
            (WikiRoute::Index.path(), scope.to_string()),
        )),
        &title_row(PageKind::DomainIndex, &page.title, None, &[], &page.scope),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "domain-index",
        &page.title,
        &index_breadcrumb(scope),
        &container,
        "",
    )
}

fn domain_margin(page: &DomainIndexPage) -> String {
    let count = counted(page.authored_group_count, "group", "groups");
    if page.authored_group_count == 0 {
        format!("<h3 class=\"margin-head\">Domains</h3><p class=\"prose\">{count}.</p>")
    } else {
        format!(
            "<h3 class=\"margin-head\">Domains</h3><p class=\"prose\">{count}. Rules inherit every Domain represented by their upstream requirements.</p>"
        )
    }
}

fn push_all_records(
    html: &mut String,
    links: &PageLinksRenderer,
    heading: &str,
    entries: &[SearchEntry],
    rules: bool,
) {
    writeln!(html, "<section><h2>{}</h2>", escape_html(heading))
        .expect("writing to a String should not fail");
    if entries.is_empty() {
        let noun = if rules { "rules" } else { "requirements" };
        writeln!(html, "<p class=\"empty-note\">No {noun} are recorded.</p>")
            .expect("writing to a String should not fail");
    } else {
        html.push_str("<ol class=\"search-results\">\n");
        for entry in entries {
            html.push_str("<li>\n");
            if rules {
                html.push_str(&icon_svg("i-shield"));
            }
            writeln!(
                html,
                "{}<p>{}</p></li>",
                links.link(&entry.link, None),
                escape_html(&entry.statement)
            )
            .expect("writing to a String should not fail");
        }
        html.push_str("</ol>\n");
    }
    html.push_str("</section>\n");
}

fn push_group(
    html: &mut String,
    links: &PageLinksRenderer,
    heading: &str,
    class_name: &str,
    group: &[PageLink],
) {
    if group.is_empty() {
        return;
    }
    writeln!(
        html,
        "<div class=\"domain-records {class_name}\"><h3>{}</h3>{}</div>",
        escape_html(heading),
        links.link_list(group)
    )
    .expect("writing to a String should not fail");
}
