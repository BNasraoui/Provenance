use crate::wiki::model::{PageKind, Topic, TopicIndexPage};
use std::fmt::Write as _;

use super::super::chrome::{container_html, index_breadcrumb, page_shell, title_row};
use super::super::html::{escape_attr, escape_html, link_list};
use super::super::routes::{topic_anchor, WikiRoute, UNASSIGNED_TOPIC_ANCHOR};

pub fn render_topics(scope: &str, page: &TopicIndexPage) -> String {
    let mut main = String::new();
    if page.groups.is_empty() {
        main.push_str(
            "<p class=\"empty-note\">No domains, requirements, or rules are recorded in this scope.</p>\n",
        );
    }
    for group in &page.groups {
        let (anchor, name, domain_id, description, gap) = match &group.topic {
            Topic::Defined {
                id,
                name,
                description,
            } => (
                topic_anchor(id),
                name.as_str(),
                Some(id.as_str()),
                description.as_deref(),
                false,
            ),
            Topic::Missing { id } => (
                topic_anchor(id),
                "Missing domain",
                Some(id.as_str()),
                None,
                true,
            ),
            Topic::Unassigned => (
                UNASSIGNED_TOPIC_ANCHOR.to_string(),
                "Unassigned",
                None,
                Some("Records with no domain, or rules with no domain-backed requirement provenance."),
                true,
            ),
        };
        writeln!(
            main,
            "<section class=\"topic-group{}\" id=\"{}\">\n<h2>{}</h2>",
            if gap { " topic-gap" } else { "" },
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
                "<p class=\"data-note\">Domain metadata is missing or unassigned; membership shown here follows the available provenance.</p>\n",
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
        "<h3 class=\"margin-head\">Domains</h3><p class=\"prose\">{} groups. Rules inherit every domain represented by their upstream requirements.</p>",
        page.groups.len()
    );
    let container = container_html(
        Some((
            PageKind::ScopeIndex,
            (WikiRoute::Index.path(), scope.to_string()),
        )),
        &title_row(PageKind::TopicIndex, &page.title, None, &[], &page.scope),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "topic-index",
        &page.title,
        &index_breadcrumb(scope),
        &container,
        "",
    )
}

fn push_group(
    html: &mut String,
    heading: &str,
    class_name: &str,
    links: &[crate::wiki::model::PageLink],
) {
    if links.is_empty() {
        return;
    }
    writeln!(
        html,
        "<div class=\"topic-records {class_name}\"><h3>{}</h3>{}</div>",
        escape_html(heading),
        link_list(links)
    )
    .expect("writing to a String should not fail");
}
