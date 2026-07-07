use crate::wiki::model::{DecisionSection, LineageEntry, PageLink, RuleCard};
use std::fmt::Write as _;

use super::html::{escape_attr, escape_html, evidence_html, icon_svg, link_html, link_list};
use super::labels::{capitalize, format_date_ms, modality_word, sev_chip, severity_word};

pub(in crate::wiki::render) fn push_section_open(
    html: &mut String,
    head_class: &str,
    head_icon: Option<&str>,
    head: &str,
) {
    let class = if head_class.is_empty() {
        "section-head".to_string()
    } else {
        format!("section-head {head_class}")
    };
    let icon = head_icon.map(icon_svg).unwrap_or_default();
    write!(
        html,
        "<section>\n<h2 class=\"{class}\">{icon}{}</h2>\n",
        escape_html(head)
    )
    .expect("writing to a String should not fail");
}

pub(in crate::wiki::render) fn push_prose_section(
    html: &mut String,
    head_class: &str,
    head_icon: Option<&str>,
    head: &str,
    body: &str,
) {
    push_section_open(html, head_class, head_icon, head);
    write!(
        html,
        "<p class=\"prose\">{}</p>\n</section>\n",
        escape_html(body)
    )
    .expect("writing to a String should not fail");
}

pub(in crate::wiki::render) fn push_decision_sections(
    html: &mut String,
    decision: &DecisionSection,
) {
    push_section_open(html, "sh-resolution", Some("i-scale"), "Resolving Decision");
    write!(
        html,
        "<h3 class=\"decision-title\">{}</h3>\n<blockquote class=\"position\">{}</blockquote>\n</section>\n",
        link_html(&decision.link),
        escape_html(&decision.position)
    )
    .expect("writing to a String should not fail");
    push_prose_section(
        html,
        "sh-resolution",
        None,
        "Rationale",
        &decision.rationale,
    );
    if let Some(context) = &decision.context {
        push_prose_section(html, "sh-resolution", None, "Context", context);
    }
}

pub(in crate::wiki::render) fn push_rule_territory_card(html: &mut String, rules: &[RuleCard]) {
    html.push_str("<div class=\"territory-card rule\">\n");
    write!(
        html,
        "<div class=\"card-head\">{}Produced Rules — {}</div>\n<ul class=\"rule-list\">\n",
        icon_svg("i-book-open"),
        rules.len()
    )
    .expect("writing to a String should not fail");
    for rule in rules {
        html.push_str("<li>\n");
        writeln!(
            html,
            "<span class=\"rcode\"><a href=\"{}\">{}</a></span>",
            escape_attr(&rule.link.target.route()),
            escape_html(&rule.rule_code)
        )
        .expect("writing to a String should not fail");
        writeln!(
            html,
            "<span class=\"rname\">{}</span>",
            escape_html(rule.name.as_deref().unwrap_or(&rule.rule_code))
        )
        .expect("writing to a String should not fail");
        html.push_str("<span class=\"rmeta\">");
        html.push_str(&sev_chip(
            severity_word(&rule.severity),
            severity_word(&rule.severity),
        ));
        if let Some(modality) = &rule.modality {
            html.push_str(&sev_chip("modality", modality_word(modality)));
        }
        html.push_str("</span>\n");
        writeln!(
            html,
            "<span class=\"rstatement\">{}</span>",
            escape_html(&rule.statement)
        )
        .expect("writing to a String should not fail");
        if !rule.evidence.is_empty() {
            let refs: Vec<String> = rule.evidence.iter().map(evidence_html).collect();
            writeln!(html, "<span class=\"rref\">{}</span>", refs.join(" · "))
                .expect("writing to a String should not fail");
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul>\n</div>\n");
}

pub(in crate::wiki::render) fn push_attribution(
    html: &mut String,
    made_by: Option<&str>,
    approved_by: Option<&str>,
    approved_at: Option<i64>,
    status_word: &str,
) {
    html.push_str("<section class=\"attribution\" aria-label=\"Attribution\">\n");
    if let Some(made_by) = made_by {
        push_attribution_row(html, "i-user", "Made by", &escape_html(made_by));
    }
    if let Some(approved_by) = approved_by {
        push_attribution_row(html, "i-user", "Approved by", &escape_html(approved_by));
    }
    if let Some(approved_at) = approved_at {
        push_attribution_row(html, "i-calendar", "Approved", &format_date_ms(approved_at));
    }
    push_attribution_row(
        html,
        "i-check-circle",
        "Decision status",
        &capitalize(status_word),
    );
    html.push_str("</section>\n");
}

pub(in crate::wiki::render) fn push_attribution_row(
    html: &mut String,
    icon: &str,
    key: &str,
    value_html: &str,
) {
    writeln!(
        html,
        "<div>{}<span class=\"k\">{key}</span><span class=\"v\">{value_html}</span></div>",
        icon_svg(icon)
    )
    .expect("writing to a String should not fail");
}

pub(in crate::wiki::render) fn push_classification_block(html: &mut String, rows: &str) {
    if rows.is_empty() {
        return;
    }
    write!(
        html,
        "<div class=\"classification\">\n<h3 class=\"margin-head\">Classification</h3>\n{rows}</div>\n"
    )
    .expect("writing to a String should not fail");
}

pub(in crate::wiki::render) fn push_classification_row(
    html: &mut String,
    icon: &str,
    key: &str,
    value: &str,
    mono: bool,
) {
    let class = if mono { "v mono" } else { "v" };
    writeln!(
        html,
        "<div class=\"row\">{}<span class=\"k\">{key}</span><span class=\"{class}\">{}</span></div>",
        icon_svg(icon),
        escape_html(value)
    )
    .expect("writing to a String should not fail");
}

pub(in crate::wiki::render) fn push_classification_link_row(
    html: &mut String,
    icon: &str,
    key: &str,
    link: &PageLink,
) {
    writeln!(
        html,
        "<div class=\"row\">{}<span class=\"k\">{key}</span><span class=\"v\">{}</span></div>",
        icon_svg(icon),
        link_html(link)
    )
    .expect("writing to a String should not fail");
}

pub(in crate::wiki::render) fn push_lineage(html: &mut String, lineage: &[LineageEntry]) {
    if lineage.is_empty() {
        return;
    }
    html.push_str("<div class=\"lineage\">\n<h3 class=\"margin-head\">Lineage</h3>\n<ol>\n");
    for entry in lineage {
        if entry.is_current {
            writeln!(
                html,
                "<li class=\"current\">{}</li>",
                escape_html(&entry.link.title)
            )
            .expect("writing to a String should not fail");
        } else {
            writeln!(html, "<li>{}</li>", link_html(&entry.link))
                .expect("writing to a String should not fail");
        }
    }
    html.push_str("</ol>\n</div>\n");
}

pub(in crate::wiki::render) fn push_orphan_group(
    html: &mut String,
    head: &str,
    links: &[PageLink],
) {
    if links.is_empty() {
        return;
    }
    write!(html, "<h3>{}</h3>\n{}", escape_html(head), link_list(links))
        .expect("writing to a String should not fail");
}
