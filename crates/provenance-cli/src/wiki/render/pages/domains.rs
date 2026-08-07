use crate::wiki::model::{DomainIndexPage, DomainState, PageKind, PageLink};
use crate::wiki::routes::{domain_anchor, WikiRoute, UNASSIGNED_DOMAIN_ANCHOR};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::html::{escape_attr, escape_html, link_list};

pub fn render_domains(scope: &str, page: &DomainIndexPage) -> String {
    let mut main = String::new();
    if page.groups.is_empty() {
        main.push_str(
            "<p class=\"empty-note\">No domains, requirements, or rules are recorded in this scope.</p>\n",
        );
    }
    for group in &page.groups {
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
                "Requirements",
                "requirement",
                &group.requirements,
            );
            push_group(&mut main, "Rules", "rule", &group.rules);
        }
        main.push_str("</section>\n");
    }
    let margin = format!(
        "<h3 class=\"margin-head\">Domains</h3><p class=\"prose\">{} groups. Rules inherit every Domain represented by their upstream requirements.</p>",
        page.groups.len()
    );
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

fn push_group(html: &mut String, heading: &str, class_name: &str, links: &[PageLink]) {
    if links.is_empty() {
        return;
    }
    writeln!(
        html,
        "<div class=\"domain-records {class_name}\"><h3>{}</h3>{}</div>",
        escape_html(heading),
        link_list(links)
    )
    .expect("writing to a String should not fail");
}
