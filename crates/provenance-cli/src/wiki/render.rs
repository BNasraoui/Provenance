//! Renders the wiki page model to semantic HTML.
//!
//! Pages follow the design mockup: type-colored accent bar, top chrome with
//! wordmark and breadcrumb, display-serif title with type and status badges,
//! a 780px body next to a 220px scholarly margin (numbered citations, dashed
//! gap notices, classification rows, lineage chain), and a full-width Field
//! Notes band. Everything is a pure `model -> String` function; the CSS and
//! theme switcher are vendored in [`crate::wiki::theme`].

use crate::wiki::links::{EvidenceRef, InlineRef};
use crate::wiki::model::{
    DecisionSection, EvidenceThread, GapNotice, InputCitation, LineageEntry, PageId, PageKind,
    PageLink, RequirementPage, ResolutionPage, RuleCard, RulePage, ScopeIndexPage, SourceCitation,
    SourcePage, WikiCorpus,
};
use crate::wiki::theme::{ICON_DEFS, THEME_SCRIPT};
use provenance_core::{
    MessageRole, NodeType, RequirementStatus, ResolutionInputType, ResolutionStatus, RuleModality,
    RuleSeverity, RuleStatus, RuleType, SourceType, ThreadStatus,
};
use std::fmt::Write as _;

/// Route the vendored stylesheet is referenced under; the server (or static
/// build) must expose [`crate::wiki::theme::WIKI_CSS`] here.
pub const WIKI_CSS_ROUTE: &str = "/assets/provenance-wiki.css";

/// One rendered page: its canonical route, title, and full HTML document.
#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub route: String,
    pub title: String,
    pub html: String,
}

/// Renders every page in the corpus, index first, then requirements,
/// resolutions, rules, and sources in record order.
pub fn render_corpus(corpus: &WikiCorpus) -> Vec<RenderedPage> {
    let scope = corpus.scope.as_str();
    let mut pages = vec![rendered(
        &corpus.index.id,
        &corpus.index.title,
        render_index(scope, &corpus.index),
    )];
    for page in &corpus.requirements {
        pages.push(rendered(
            &page.id,
            &page.title,
            render_requirement(scope, page),
        ));
    }
    for page in &corpus.resolutions {
        pages.push(rendered(
            &page.id,
            &page.title,
            render_resolution(scope, page),
        ));
    }
    for page in &corpus.rules {
        pages.push(rendered(&page.id, &page.title, render_rule(scope, page)));
    }
    for page in &corpus.sources {
        pages.push(rendered(&page.id, &page.title, render_source(scope, page)));
    }
    pages
}

fn rendered(id: &PageId, title: &str, html: String) -> RenderedPage {
    RenderedPage {
        route: id.route(),
        title: title.to_string(),
        html,
    }
}

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
            main.push_str(&status_badge(requirement_status_word(&entry.status)));
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

/// Renders a requirement detail page.
#[allow(clippy::too_many_lines)]
pub fn render_requirement(scope: &str, page: &RequirementPage) -> String {
    let mut main = String::new();
    push_prose_section(
        &mut main,
        "sh-requirement",
        None,
        "Statement",
        &page.statement,
    );
    if let Some(description) = &page.description {
        push_prose_section(
            &mut main,
            "sh-requirement",
            None,
            "Description",
            description,
        );
    }
    if let Some(fog) = &page.fog {
        push_prose_section(&mut main, "", None, "Fog", fog);
    }
    for decision in &page.decisions {
        push_decision_sections(&mut main, decision);
    }
    if !page.produced_rules.is_empty() {
        push_section_open(&mut main, "sh-resolution", None, "Downstream Territory");
        main.push_str("<div class=\"territory\">\n");
        push_rule_territory_card(&mut main, &page.produced_rules);
        main.push_str("</div>\n</section>\n");
    }
    if !page.children.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Refined Into",
        );
        main.push_str("<div class=\"territory\">\n<div class=\"territory-card requirement\">\n");
        writeln!(
            main,
            "<div class=\"card-head\">{}Refinements — {}</div>",
            icon_svg("i-git-branch"),
            page.children.len()
        )
        .expect("writing to a String should not fail");
        main.push_str(&link_list(&page.children));
        main.push_str("</div>\n</div>\n</section>\n");
    }
    for decision in &page.decisions {
        push_attribution(
            &mut main,
            decision.made_by.as_deref(),
            decision.approved_by.as_deref(),
            decision.approved_at,
            resolution_status_word(&decision.status),
        );
    }

    let mut margin = String::new();
    margin.push_str("<h3 class=\"margin-head\">Sources</h3>\n");
    push_gap_citations(&mut margin, &page.gaps);
    push_source_citations(&mut margin, &page.sources);
    let mut rows = String::new();
    if let Some(domain_id) = &page.domain_id {
        push_classification_row(&mut rows, "i-git-branch", "Domain", domain_id, true);
    }
    for decision in &page.decisions {
        if let Some(enforcement) = &decision.enforcement {
            push_classification_row(&mut rows, "i-shield", "Enforcement", enforcement, false);
        }
        if let Some(confidence) = decision.confidence {
            push_classification_row(
                &mut rows,
                "i-gauge",
                "Confidence",
                &format_confidence(confidence),
                false,
            );
        }
        push_classification_row(
            &mut rows,
            "i-scale",
            "Decision",
            &decision.link.target.record_id,
            true,
        );
    }
    push_classification_block(&mut margin, &rows);
    push_lineage(&mut margin, &page.lineage);

    let back = page.back_link.as_ref().map_or_else(
        || ("/".to_string(), scope.to_string()),
        |link| (link.target.route(), link.title.clone()),
    );
    let container = container_html(
        Some((PageKind::Requirement, back)),
        &title_row(
            PageKind::Requirement,
            &page.title,
            Some(&status_badge(requirement_status_word(&page.status))),
            &[],
            &page.id.record_id,
        ),
        &main,
        &margin,
    );
    page_shell(
        scope,
        "requirement",
        &page.title,
        &breadcrumb_from_lineage(&page.lineage),
        &container,
        &field_notes(&page.threads, &page.id),
    )
}

/// Renders a resolution detail page.
pub fn render_resolution(scope: &str, page: &ResolutionPage) -> String {
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
        main.push_str(&link_list(&page.resolves));
        main.push_str("</section>\n");
    }
    if !page.spawned.is_empty() {
        push_section_open(
            &mut main,
            "sh-requirement",
            Some("i-git-branch"),
            "Spawned Requirements",
        );
        main.push_str(&link_list(&page.spawned));
        main.push_str("</section>\n");
    }
    if !page.produced_rules.is_empty() {
        push_section_open(&mut main, "sh-resolution", None, "Downstream Territory");
        main.push_str("<div class=\"territory\">\n");
        push_rule_territory_card(&mut main, &page.produced_rules);
        main.push_str("</div>\n</section>\n");
    }
    push_attribution(
        &mut main,
        page.made_by.as_deref(),
        page.approved_by.as_deref(),
        page.approved_at,
        resolution_status_word(&page.status),
    );

    let mut margin = String::new();
    margin.push_str("<h3 class=\"margin-head\">Inputs</h3>\n");
    push_gap_citations(&mut margin, &page.gaps);
    push_input_citations(&mut margin, &page.inputs);
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
        push_classification_link_row(&mut rows, "i-scale", "Superseded by", superseded_by);
    }
    push_classification_block(&mut margin, &rows);

    let container = container_html(
        Some((PageKind::Resolution, ("/".to_string(), scope.to_string()))),
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
        Some((PageKind::Rule, ("/".to_string(), scope.to_string()))),
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

/// Renders the not-found page served for unknown routes.
pub fn render_not_found(scope: &str, path: &str) -> String {
    let container = format!(
        "<div class=\"title-row\">\n<div>\n<h1>Page not found</h1>\n</div>\n</div>\n\
         <div class=\"body-grid\">\n<div class=\"body-main\">\n\
         <p class=\"prose\">No wiki page is served at <code>{}</code>.</p>\n\
         <p class=\"prose\"><a href=\"/\">Back to the scope index.</a></p>\n\
         </div>\n</div>\n",
        escape_html(path)
    );
    page_shell(
        scope,
        "scope-index",
        "Page not found",
        &index_breadcrumb(scope),
        &container,
        "",
    )
}

// ---- building blocks ----

fn page_shell(
    scope: &str,
    kind_class: &str,
    title: &str,
    breadcrumb: &str,
    container: &str,
    field_notes: &str,
) -> String {
    let mut html = String::new();
    html.push_str("<!doctype html>\n<html lang=\"en\" data-theme=\"statesman\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    writeln!(
        html,
        "<title>{} - Provenance Wiki</title>",
        escape_html(title)
    )
    .expect("writing to a String should not fail");
    writeln!(html, "<link rel=\"stylesheet\" href=\"{WIKI_CSS_ROUTE}\">")
        .expect("writing to a String should not fail");
    html.push_str("</head>\n<body>\n");
    html.push_str(ICON_DEFS.trim_start_matches('\n'));
    html.push_str("<header class=\"chrome\">\n<div class=\"chrome-inner\">\n");
    writeln!(
        html,
        "<span class=\"wordmark\">Provenance<span class=\"scope\">{}</span></span>",
        escape_html(scope)
    )
    .expect("writing to a String should not fail");
    writeln!(html, "<nav aria-label=\"Breadcrumb\">{breadcrumb}</nav>")
        .expect("writing to a String should not fail");
    html.push_str(
        "<label class=\"theme-select\">Theme\n<select id=\"theme-select\">\n\
         <option value=\"statesman\" selected>Statesman</option>\n\
         <option value=\"piano\">Piano</option>\n\
         <option value=\"latte\">Catppuccin Latte</option>\n\
         <option value=\"mocha\">Catppuccin Mocha</option>\n\
         <option value=\"dracula\">Dracula</option>\n\
         </select>\n</label>\n",
    );
    html.push_str("</div>\n</header>\n");
    writeln!(
        html,
        "<div class=\"accent-bar {kind_class}\" aria-hidden=\"true\"></div>"
    )
    .expect("writing to a String should not fail");
    write!(html, "<div class=\"container\">\n{container}</div>\n")
        .expect("writing to a String should not fail");
    html.push_str(field_notes);
    html.push_str("<script>");
    html.push_str(THEME_SCRIPT);
    html.push_str("</script>\n</body>\n</html>\n");
    html
}

fn container_html(
    back: Option<(PageKind, (String, String))>,
    title_row: &str,
    main: &str,
    margin: &str,
) -> String {
    let mut html = String::new();
    if let Some((kind, (route, title))) = back {
        write!(
            html,
            "<a class=\"back-link {}\" href=\"{}\">\n{}{}\n</a>\n",
            kind_class(kind),
            escape_attr(&route),
            icon_svg("i-arrow-left"),
            escape_html(&title)
        )
        .expect("writing to a String should not fail");
    }
    html.push_str(title_row);
    write!(
        html,
        "<div class=\"body-grid\">\n<div class=\"body-main\">\n{main}</div>\n\
         <aside class=\"body-margin\">\n{margin}</aside>\n</div>\n"
    )
    .expect("writing to a String should not fail");
    html
}

fn title_row(
    kind: PageKind,
    title: &str,
    status_badge: Option<&str>,
    chips: &[String],
    id_chip: &str,
) -> String {
    let mut html = String::new();
    write!(
        html,
        "<div class=\"title-row\">\n<svg class=\"icon {}\" aria-hidden=\"true\"><use href=\"#{}\"/></svg>\n<div>\n<h1>{}</h1>\n<div class=\"badge-row\">\n",
        kind_class(kind),
        kind_icon(kind),
        escape_html(title)
    )
    .expect("writing to a String should not fail");
    if kind != PageKind::ScopeIndex {
        writeln!(
            html,
            "<span class=\"type-badge {}\">{}{}</span>",
            kind_class(kind),
            icon_svg(kind_icon(kind)),
            kind_label(kind)
        )
        .expect("writing to a String should not fail");
    }
    if let Some(badge) = status_badge {
        html.push_str(badge);
        html.push('\n');
    }
    for chip in chips {
        html.push_str(chip);
        html.push('\n');
    }
    writeln!(
        html,
        "<span class=\"id-chip\">{}</span>",
        escape_html(id_chip)
    )
    .expect("writing to a String should not fail");
    html.push_str("</div>\n</div>\n</div>\n");
    html
}

fn push_section_open(html: &mut String, head_class: &str, head_icon: Option<&str>, head: &str) {
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

fn push_prose_section(
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

fn push_decision_sections(html: &mut String, decision: &DecisionSection) {
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

fn push_rule_territory_card(html: &mut String, rules: &[RuleCard]) {
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

fn push_attribution(
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

fn push_attribution_row(html: &mut String, icon: &str, key: &str, value_html: &str) {
    writeln!(
        html,
        "<div>{}<span class=\"k\">{key}</span><span class=\"v\">{value_html}</span></div>",
        icon_svg(icon)
    )
    .expect("writing to a String should not fail");
}

fn push_gap_citations(html: &mut String, gaps: &[GapNotice]) {
    for gap in gaps {
        write!(
            html,
            "<div class=\"citation gap\">\n<div class=\"cite-head\"><span class=\"cite-type\" style=\"color:inherit\">Gap</span></div>\n<p>{}</p>\n</div>\n",
            escape_html(&gap.detail)
        )
        .expect("writing to a String should not fail");
    }
}

fn push_source_citations(html: &mut String, sources: &[SourceCitation]) {
    for (index, citation) in sources.iter().enumerate() {
        write!(
            html,
            "<div class=\"citation\">\n<div class=\"cite-head\"><span class=\"cite-num\">[{}]</span><span class=\"cite-type\">{}</span></div>\n",
            index + 1,
            source_type_label(&citation.source_type)
        )
        .expect("writing to a String should not fail");
        let clause = citation
            .clause
            .as_ref()
            .map(|clause| format!(" — {}", escape_html(clause)))
            .unwrap_or_default();
        writeln!(html, "<p>{}{clause}</p>", link_html(&citation.link))
            .expect("writing to a String should not fail");
        if let Some(reference) = &citation.reference {
            writeln!(
                html,
                "<p class=\"cite-ref\">{}</p>",
                evidence_html(reference)
            )
            .expect("writing to a String should not fail");
        }
        html.push_str("</div>\n");
    }
}

fn push_input_citations(html: &mut String, inputs: &[InputCitation]) {
    for (index, citation) in inputs.iter().enumerate() {
        write!(
            html,
            "<div class=\"citation\">\n<div class=\"cite-head\"><span class=\"cite-num\">[{}]</span><span class=\"cite-type\">{}</span></div>\n<p>{}</p>\n<p class=\"cite-ref\">{}</p>\n</div>\n",
            index + 1,
            input_type_label(&citation.input_type),
            escape_html(&citation.summary),
            evidence_html(&citation.reference)
        )
        .expect("writing to a String should not fail");
    }
}

fn push_classification_block(html: &mut String, rows: &str) {
    if rows.is_empty() {
        return;
    }
    write!(
        html,
        "<div class=\"classification\">\n<h3 class=\"margin-head\">Classification</h3>\n{rows}</div>\n"
    )
    .expect("writing to a String should not fail");
}

fn push_classification_row(html: &mut String, icon: &str, key: &str, value: &str, mono: bool) {
    let class = if mono { "v mono" } else { "v" };
    writeln!(
        html,
        "<div class=\"row\">{}<span class=\"k\">{key}</span><span class=\"{class}\">{}</span></div>",
        icon_svg(icon),
        escape_html(value)
    )
    .expect("writing to a String should not fail");
}

fn push_classification_link_row(html: &mut String, icon: &str, key: &str, link: &PageLink) {
    writeln!(
        html,
        "<div class=\"row\">{}<span class=\"k\">{key}</span><span class=\"v\">{}</span></div>",
        icon_svg(icon),
        link_html(link)
    )
    .expect("writing to a String should not fail");
}

fn push_lineage(html: &mut String, lineage: &[LineageEntry]) {
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

fn push_orphan_group(html: &mut String, head: &str, links: &[PageLink]) {
    if links.is_empty() {
        return;
    }
    write!(html, "<h3>{}</h3>\n{}", escape_html(head), link_list(links))
        .expect("writing to a String should not fail");
}

fn field_notes(threads: &[EvidenceThread], page_id: &PageId) -> String {
    if threads.is_empty() {
        return String::new();
    }
    let mut html = String::new();
    html.push_str("<div class=\"field-notes\">\n<div class=\"container\">\n");
    for thread in threads {
        let borrowed = if thread.parent_id == page_id.record_id {
            String::new()
        } else {
            format!(
                " · on {} {}",
                node_type_word(thread.parent_type),
                escape_html(&thread.parent_id)
            )
        };
        write!(
            html,
            "<div class=\"fn-head\">\n{}<h2>Field Notes</h2>\n<span class=\"count\">{} · {}</span>\n<span class=\"thread-id\">{}{borrowed}</span>\n</div>\n",
            icon_svg("i-message-square"),
            counted(thread.messages.len(), "message", "messages"),
            thread_status_word(&thread.status),
            escape_html(&thread.thread_id)
        )
        .expect("writing to a String should not fail");
        html.push_str("<div class=\"fn-list\">\n");
        for note in &thread.messages {
            let (role_icon, role_label) = match note.role {
                MessageRole::User => ("i-user", "Human"),
                MessageRole::Assistant => ("i-bot", "Agent"),
                MessageRole::System => ("i-bot", "System"),
            };
            // The record model carries no author-name field, only `role`
            // (human/agent/system) and the message's own stable id. Showing
            // that id here would read as a person's/agent's name, so `.who`
            // falls back to the same readable role label as the badge next
            // to it rather than an opaque `msg_...` id.
            write!(
                html,
                "<div class=\"field-note\">\n<span class=\"role-badge\">{}{role_label}</span>\n<div class=\"fn-body\">\n<div class=\"fn-meta\"><span class=\"who\">{role_label}</span><time datetime=\"{}\">{}</time></div>\n<p class=\"fn-content\">{}</p>\n</div>\n</div>\n",
                icon_svg(role_icon),
                format_date_iso_ms(note.created_at),
                format_date_ms(note.created_at),
                linkify_body(&note.body, &note.refs)
            )
            .expect("writing to a String should not fail");
        }
        html.push_str("</div>\n");
    }
    html.push_str("</div>\n</div>\n");
    html
}

fn breadcrumb_from_lineage(lineage: &[LineageEntry]) -> String {
    let ancestors: Vec<String> = lineage
        .iter()
        .filter(|entry| !entry.is_current)
        .map(|entry| link_html(&entry.link))
        .collect();
    ancestors.join(" <span class=\"sep\">›</span> ")
}

fn index_breadcrumb(scope: &str) -> String {
    format!("<a href=\"/\">{}</a>", escape_html(scope))
}

fn link_html(link: &PageLink) -> String {
    format!(
        "<a href=\"{}\">{}</a>",
        escape_attr(&link.target.route()),
        escape_html(&link.title)
    )
}

fn link_list(links: &[PageLink]) -> String {
    let mut html = String::from("<ul class=\"link-list\">\n");
    for link in links {
        writeln!(html, "<li>{}</li>", link_html(link))
            .expect("writing to a String should not fail");
    }
    html.push_str("</ul>\n");
    html
}

fn evidence_html(evidence: &EvidenceRef) -> String {
    evidence.href.as_ref().map_or_else(
        || escape_html(&evidence.label),
        |href| {
            format!(
                "<a href=\"{}\">{}</a>",
                escape_attr(href),
                escape_html(&evidence.label)
            )
        },
    )
}

fn icon_svg(symbol: &str) -> String {
    format!("<svg class=\"icon\"><use href=\"#{symbol}\"/></svg>")
}

fn status_badge(word: &str) -> String {
    let icon = match word {
        "approved" | "resolved" | "active" => "i-check-circle",
        _ => "i-search",
    };
    format!(
        "<span class=\"status-badge {word}\">{}{}</span>",
        icon_svg(icon),
        capitalize(word)
    )
}

fn sev_chip(class_word: &str, label: &str) -> String {
    format!("<span class=\"sev {class_word}\">{label}</span>")
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

fn counted(count: usize, singular: &str, plural: &str) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
}

const fn kind_class(kind: PageKind) -> &'static str {
    match kind {
        PageKind::ScopeIndex => "scope-index",
        PageKind::Requirement => "requirement",
        PageKind::Resolution => "resolution",
        PageKind::Rule => "rule",
        PageKind::Source => "source",
    }
}

const fn kind_label(kind: PageKind) -> &'static str {
    match kind {
        PageKind::ScopeIndex => "Scope",
        PageKind::Requirement => "Requirement",
        PageKind::Resolution => "Resolution",
        PageKind::Rule => "Rule",
        PageKind::Source => "Source",
    }
}

const fn kind_icon(kind: PageKind) -> &'static str {
    match kind {
        PageKind::ScopeIndex | PageKind::Rule | PageKind::Source => "i-book-open",
        PageKind::Requirement => "i-git-branch",
        PageKind::Resolution => "i-scale",
    }
}

const fn requirement_status_word(status: &RequirementStatus) -> &'static str {
    match status {
        RequirementStatus::Active => "active",
        RequirementStatus::Discovery => "discovery",
        RequirementStatus::Refinement => "refinement",
        RequirementStatus::Resolved => "resolved",
    }
}

const fn resolution_status_word(status: &ResolutionStatus) -> &'static str {
    match status {
        ResolutionStatus::Draft => "draft",
        ResolutionStatus::Review => "review",
        ResolutionStatus::Proposed => "proposed",
        ResolutionStatus::Approved => "approved",
        ResolutionStatus::Rejected => "rejected",
        ResolutionStatus::Revised => "revised",
        ResolutionStatus::Superseded => "superseded",
        ResolutionStatus::Abandoned => "abandoned",
    }
}

const fn rule_status_word(status: &RuleStatus) -> &'static str {
    match status {
        RuleStatus::Draft => "draft",
        RuleStatus::Review => "review",
        RuleStatus::Active => "active",
        RuleStatus::Deprecated => "deprecated",
        RuleStatus::Archived => "archived",
    }
}

const fn thread_status_word(status: &ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Active => "active",
        ThreadStatus::Resolved => "resolved",
        ThreadStatus::Archived => "archived",
    }
}

const fn severity_word(severity: &RuleSeverity) -> &'static str {
    match severity {
        RuleSeverity::Low => "low",
        RuleSeverity::Medium => "medium",
        RuleSeverity::High => "high",
        RuleSeverity::Critical => "critical",
    }
}

const fn modality_word(modality: &RuleModality) -> &'static str {
    match modality {
        RuleModality::Obligation => "obligation",
        RuleModality::Prohibition => "prohibition",
        RuleModality::Necessity => "necessity",
    }
}

const fn rule_type_word(rule_type: &RuleType) -> &'static str {
    match rule_type {
        RuleType::Business => "business",
        RuleType::Functional => "functional",
        RuleType::Technical => "technical",
    }
}

const fn source_type_label(source_type: &SourceType) -> &'static str {
    match source_type {
        SourceType::Policy => "Policy",
        SourceType::Document => "Document",
        SourceType::Legislation => "Legislation",
        SourceType::CompanyAgreement => "Company agreement",
        SourceType::SystemState => "System state",
        SourceType::ExternalIntegration => "External integration",
        SourceType::DomainKnowledge => "Domain knowledge",
        SourceType::ProjectArtifact => "Project artifact",
        SourceType::Incident => "Incident",
        SourceType::ApiSpec => "API spec",
    }
}

const fn input_type_label(input_type: &ResolutionInputType) -> &'static str {
    match input_type {
        ResolutionInputType::Regulatory => "Regulatory",
        ResolutionInputType::LegalAdvice => "Legal advice",
        ResolutionInputType::Commercial => "Commercial",
        ResolutionInputType::Benchmark => "Benchmark",
        ResolutionInputType::Technical => "Technical",
        ResolutionInputType::Incident => "Incident",
        ResolutionInputType::SourceMaterial => "Source material",
    }
}

const fn node_type_word(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Source => "source",
        NodeType::Requirement => "requirement",
        NodeType::Resolution => "resolution",
        NodeType::Rule => "rule",
        NodeType::Topic => "topic",
        NodeType::Question => "question",
    }
}

fn format_date_iso_ms(ms: i64) -> String {
    let (year, month, day) = civil_from_days(ms.div_euclid(86_400_000));
    format!("{year:04}-{month:02}-{day:02}")
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(text: &str) -> String {
    escape_html(text).replace('"', "&quot;")
}

/// Formats an epoch-milliseconds timestamp as a civil UTC date, mockup
/// style: `18 Apr 2026`.
fn format_date_ms(ms: i64) -> String {
    let days = ms.div_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days);
    let month = match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        _ => "Dec",
    };
    format!("{day} {month} {year}")
}

/// Days since 1970-01-01 to a proleptic Gregorian date (Howard Hinnant's
/// `civil_from_days` algorithm).
const fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn format_confidence(confidence: f64) -> String {
    format!("{}%", (confidence * 100.0).round() as u32)
}

/// Escapes a field-note body while wrapping each [`InlineRef`] span in an
/// anchor. Spans are byte offsets into `body`, non-overlapping and sorted.
fn linkify_body(body: &str, refs: &[InlineRef]) -> String {
    let mut html = String::new();
    let mut cursor = 0;
    for inline in refs {
        if inline.start < cursor || inline.end > body.len() {
            continue;
        }
        html.push_str(&escape_html(&body[cursor..inline.start]));
        write!(
            html,
            "<a class=\"src\" href=\"{}\">{}</a>",
            escape_attr(&inline.href),
            escape_html(&inline.label)
        )
        .expect("writing to a String should not fail");
        cursor = inline.end;
    }
    html.push_str(&escape_html(&body[cursor..]));
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wiki::links::InlineRef;

    #[test]
    fn escape_html_escapes_markup_characters() {
        assert_eq!(escape_html("a < b & c > d"), "a &lt; b &amp; c &gt; d");
    }

    #[test]
    fn escape_attr_also_escapes_quotes() {
        assert_eq!(
            escape_attr("say \"hi\" & <go>"),
            "say &quot;hi&quot; &amp; &lt;go&gt;"
        );
    }

    #[test]
    fn format_date_ms_renders_epoch_milliseconds_as_a_civil_date() {
        assert_eq!(format_date_ms(1_714_780_800_000), "4 May 2024");
        assert_eq!(format_date_ms(0), "1 Jan 1970");
    }

    #[test]
    fn format_date_ms_handles_dates_before_the_epoch() {
        assert_eq!(format_date_ms(-86_400_000), "31 Dec 1969");
    }

    #[test]
    fn format_confidence_renders_a_percentage() {
        assert_eq!(format_confidence(0.97), "97%");
        assert_eq!(format_confidence(1.0), "100%");
    }

    #[test]
    fn linkify_body_wraps_ref_spans_in_anchors_and_escapes_the_rest() {
        let body = "See src/UseCase.php:153-156 & friends";
        let refs = vec![InlineRef {
            start: 4,
            end: 27,
            label: "src/UseCase.php:153-156".to_string(),
            href: "https://example.test/blob".to_string(),
        }];
        assert_eq!(
            linkify_body(body, &refs),
            "See <a class=\"src\" href=\"https://example.test/blob\">src/UseCase.php:153-156</a> &amp; friends"
        );
    }

    #[test]
    fn linkify_body_escapes_plain_text_when_there_are_no_refs() {
        assert_eq!(linkify_body("a < b", &[]), "a &lt; b");
    }

    use crate::wiki::links::{EvidenceRef, LinkResolver};
    use crate::wiki::model::{
        CorpusCounts, DecisionSection, EvidenceThread, FieldNote, GapKind, GapNotice, IndexEntry,
        InputCitation, LineageEntry, OrphanReport, PageId, PageKind, PageLink, RequirementPage,
        ResolutionPage, RuleCard, RulePage, ScopeIndexPage, SourceCitation, SourcePage, WikiCorpus,
    };
    use provenance_core::{
        MessageRole, NodeType, RequirementStatus, ResolutionInputType, ResolutionStatus,
        RuleModality, RuleSeverity, RuleStatus, RuleType, SourceType, ThreadStatus,
    };

    const REMOTE: &str = "git@github.com:exampleorg/ex-api.git";

    fn link(kind: PageKind, id: &str, title: &str) -> PageLink {
        PageLink {
            target: PageId::new(kind, id),
            title: title.to_string(),
        }
    }

    fn rule_card(resolver: &LinkResolver) -> RuleCard {
        RuleCard {
            link: link(
                PageKind::Rule,
                "rule_sah_inv_016",
                "Suppress line emission for fully zero claim items",
            ),
            rule_code: "SAH-INV-016".to_string(),
            name: Some("Suppress line emission for fully zero claim items".to_string()),
            statement: "If a claim item's participant, government, and gap portions are all <= 0 \
                        after markup, no invoice lines shall be emitted for that claim item."
                .to_string(),
            status: RuleStatus::Active,
            severity: RuleSeverity::High,
            modality: Some(RuleModality::Prohibition),
            evidence: vec![resolver.resolve("src/UseCase.php:153-156")],
        }
    }

    fn field_note(resolver: &LinkResolver) -> FieldNote {
        let body = "Per-portion guard at src/UseCase.php:211-233.\n\
                    Confirmed by testCreateGapInvoiceOnly."
            .to_string();
        let refs = resolver.annotate(&body);
        FieldNote {
            message_id: "msg_000001".to_string(),
            role: MessageRole::Assistant,
            created_at: 1_714_780_800_000,
            body,
            refs,
        }
    }

    fn resolution_thread(resolver: &LinkResolver) -> EvidenceThread {
        EvidenceThread {
            thread_id: "thr_resolution_res_split_0".to_string(),
            parent_type: NodeType::Resolution,
            parent_id: "res_split".to_string(),
            status: ThreadStatus::Active,
            messages: vec![field_note(resolver)],
        }
    }

    fn decision(resolver: &LinkResolver) -> DecisionSection {
        DecisionSection {
            link: link(
                PageKind::Resolution,
                "res_split",
                "SaveInvoice per-portion split & $0 suppression extraction",
            ),
            status: ResolutionStatus::Approved,
            position: "Adopt these as 7 rules. Severity high.".to_string(),
            rationale: "Atomicity here = drift detectability.".to_string(),
            context: Some("Codebase scan of UseCase.php identified 7 patterns.".to_string()),
            enforcement: Some("Specification".to_string()),
            confidence: Some(0.97),
            inputs: vec![InputCitation {
                input_type: ResolutionInputType::Technical,
                summary: "Codebase scan — SaveInvoice use case.".to_string(),
                reference: resolver.resolve("src/UseCase.php:59-69"),
            }],
            made_by: Some("Ben Nasraoui".to_string()),
            approved_by: Some("Ben Nasraoui".to_string()),
            approved_at: Some(1_776_470_400_000),
        }
    }

    fn requirement_fixture() -> RequirementPage {
        let resolver = LinkResolver::new(Some(REMOTE));
        RequirementPage {
            id: PageId::new(PageKind::Requirement, "req_saveinvoice_split"),
            title: "SaveInvoice shall split each claim item into portions".to_string(),
            status: RequirementStatus::Discovery,
            statement: "Grouping by participant_ref with per-portion positive-amount guards."
                .to_string(),
            description: None,
            fog: None,
            domain_id: Some("dom_invoicing".to_string()),
            back_link: Some(link(
                PageKind::Requirement,
                "req_sah",
                "Support at Home (SAH)",
            )),
            lineage: vec![
                LineageEntry {
                    link: link(PageKind::Requirement, "req_platform", "ExampleOrg platform"),
                    is_current: false,
                },
                LineageEntry {
                    link: link(PageKind::Requirement, "req_sah", "Support at Home (SAH)"),
                    is_current: false,
                },
                LineageEntry {
                    link: link(
                        PageKind::Requirement,
                        "req_saveinvoice_split",
                        "SaveInvoice shall split each claim item into portions",
                    ),
                    is_current: true,
                },
            ],
            decisions: vec![decision(&resolver)],
            produced_rules: vec![rule_card(&resolver)],
            children: vec![link(
                PageKind::Requirement,
                "req_gap_lines",
                "Gap lines shall be suppressed when zero",
            )],
            sources: vec![SourceCitation {
                link: link(PageKind::Source, "source_schads", "SCHADS Award mapping"),
                source_type: SourceType::Document,
                clause: Some("clause 10.3".to_string()),
                reference: Some(resolver.resolve_at("docs/award.md", Some("abc1234"))),
            }],
            gaps: vec![],
            threads: vec![resolution_thread(&resolver)],
        }
    }

    fn gappy_requirement_fixture() -> RequirementPage {
        RequirementPage {
            id: PageId::new(PageKind::Requirement, "req_stuck"),
            title: "Rostering shall respect awards".to_string(),
            status: RequirementStatus::Resolved,
            statement: "Rostering shall respect awards.".to_string(),
            description: None,
            fog: Some("Which award clauses apply is still unclear.".to_string()),
            domain_id: None,
            back_link: None,
            lineage: vec![LineageEntry {
                link: link(
                    PageKind::Requirement,
                    "req_stuck",
                    "Rostering shall respect awards",
                ),
                is_current: true,
            }],
            decisions: vec![],
            produced_rules: vec![],
            children: vec![],
            sources: vec![],
            gaps: vec![
                GapNotice {
                    kind: GapKind::DanglingReference,
                    detail: "source ref points at source_missing, which does not exist".to_string(),
                },
                GapNotice {
                    kind: GapKind::MissingSourceRefs,
                    detail: "no source refs recorded on this requirement".to_string(),
                },
                GapNotice {
                    kind: GapKind::NoResolvingDecision,
                    detail: "resolved with no resolving decision".to_string(),
                },
            ],
            threads: vec![],
        }
    }

    fn resolution_fixture() -> ResolutionPage {
        let resolver = LinkResolver::new(Some(REMOTE));
        ResolutionPage {
            id: PageId::new(PageKind::Resolution, "res_split"),
            title: "SaveInvoice per-portion split & $0 suppression extraction".to_string(),
            status: ResolutionStatus::Approved,
            position: "Adopt these as 7 rules. Severity high.".to_string(),
            rationale: "Atomicity here = drift detectability.".to_string(),
            context: Some("Codebase scan of UseCase.php identified 7 patterns.".to_string()),
            enforcement: Some("Specification".to_string()),
            confidence: Some(0.97),
            inputs: vec![InputCitation {
                input_type: ResolutionInputType::Technical,
                summary: "Codebase scan — SaveInvoice use case.".to_string(),
                reference: resolver.resolve("src/UseCase.php:59-69"),
            }],
            made_by: Some("Ben Nasraoui".to_string()),
            approved_by: Some("Ben Nasraoui".to_string()),
            approved_at: Some(1_776_470_400_000),
            review_on: Some("2026-10-01".to_string()),
            superseded_by: None,
            resolves: vec![link(
                PageKind::Requirement,
                "req_saveinvoice_split",
                "SaveInvoice shall split each claim item into portions",
            )],
            spawned: vec![],
            produced_rules: vec![rule_card(&resolver)],
            gaps: vec![],
            threads: vec![resolution_thread(&resolver)],
        }
    }

    fn rule_fixture() -> RulePage {
        let resolver = LinkResolver::new(Some(REMOTE));
        RulePage {
            id: PageId::new(PageKind::Rule, "rule_sah_inv_016"),
            title: "Suppress line emission for fully zero claim items".to_string(),
            rule_code: "SAH-INV-016".to_string(),
            statement: "No invoice lines shall be emitted for fully zero claim items.".to_string(),
            description: None,
            status: RuleStatus::Active,
            severity: RuleSeverity::High,
            modality: Some(RuleModality::Prohibition),
            rule_type: Some(RuleType::Business),
            confidence: Some(0.92),
            extraction_method: Some("codebase_scan".to_string()),
            source_document: Some("src/UseCase.php".to_string()),
            source_section: Some("153-156".to_string()),
            evidence: vec![
                resolver.resolve("src/UseCase.php:153-156"),
                EvidenceRef {
                    label: "SCHADS Award clause 10.3".to_string(),
                    href: None,
                },
            ],
            produced_by: vec![link(
                PageKind::Resolution,
                "res_split",
                "SaveInvoice per-portion split & $0 suppression extraction",
            )],
            requirements: vec![link(
                PageKind::Requirement,
                "req_saveinvoice_split",
                "SaveInvoice shall split each claim item into portions",
            )],
            sources: vec![link(
                PageKind::Source,
                "source_schads",
                "SCHADS Award mapping",
            )],
            gaps: vec![],
            threads: vec![],
        }
    }

    fn source_fixture() -> SourcePage {
        let resolver = LinkResolver::new(Some(REMOTE));
        SourcePage {
            id: PageId::new(PageKind::Source, "source_schads"),
            title: "SCHADS Award mapping".to_string(),
            source_type: SourceType::Document,
            url: Some("https://example.test/award".to_string()),
            reference: Some(resolver.resolve_at("docs/award.md", Some("abc1234"))),
            commit_pin: Some("abc1234".to_string()),
            effective_date: Some(1_714_780_800_000),
            review_date: None,
            superseded_by: None,
            referenced_requirements: vec![link(
                PageKind::Requirement,
                "req_saveinvoice_split",
                "SaveInvoice shall split each claim item into portions",
            )],
            gaps: vec![],
            threads: vec![],
        }
    }

    fn index_fixture() -> ScopeIndexPage {
        ScopeIndexPage {
            id: PageId::new(PageKind::ScopeIndex, "default"),
            scope: "default".to_string(),
            title: "Provenance atlas — default".to_string(),
            counts: CorpusCounts {
                sources: 2,
                requirements: 3,
                resolutions: 1,
                rules: 1,
            },
            roots: vec![IndexEntry {
                link: link(PageKind::Requirement, "req_platform", "ExampleOrg platform"),
                status: RequirementStatus::Active,
                children: 2,
                resolutions: 1,
                rules: 1,
            }],
            gaps: vec![GapNotice {
                kind: GapKind::UnreferencedSource,
                detail: "source_unused is referenced by nothing".to_string(),
            }],
            orphans: OrphanReport {
                rules: vec![link(PageKind::Rule, "rule_orphan", "ORPH-001")],
                resolutions: vec![],
                sources: vec![link(PageKind::Source, "source_unused", "Unused API spec")],
            },
        }
    }

    fn corpus_fixture() -> WikiCorpus {
        WikiCorpus {
            scope: "default".to_string(),
            index: index_fixture(),
            requirements: vec![requirement_fixture(), gappy_requirement_fixture()],
            resolutions: vec![resolution_fixture()],
            rules: vec![rule_fixture()],
            sources: vec![source_fixture()],
        }
    }

    #[test]
    fn every_page_is_self_contained_and_theme_aware() {
        let corpus = corpus_fixture();
        for page in render_corpus(&corpus) {
            assert!(page.html.starts_with("<!doctype html>"), "{}", page.route);
            assert!(page.html.contains("data-theme=\"statesman\""));
            assert!(
                page.html
                    .contains("<link rel=\"stylesheet\" href=\"/assets/provenance-wiki.css\">"),
                "{}",
                page.route
            );
            for theme in ["statesman", "piano", "latte", "mocha", "dracula"] {
                assert!(
                    page.html.contains(&format!("<option value=\"{theme}\"")),
                    "{}: missing theme option {theme}",
                    page.route
                );
            }
            assert!(page.html.contains("provenance-wiki-theme"));
        }
    }

    #[test]
    fn render_corpus_routes_every_page_under_its_page_id() {
        let corpus = corpus_fixture();
        let routes: Vec<String> = render_corpus(&corpus)
            .into_iter()
            .map(|page| page.route)
            .collect();
        assert_eq!(
            routes,
            vec![
                "/".to_string(),
                "/requirements/req_saveinvoice_split/".to_string(),
                "/requirements/req_stuck/".to_string(),
                "/resolutions/res_split/".to_string(),
                "/rules/rule_sah_inv_016/".to_string(),
                "/sources/source_schads/".to_string(),
            ]
        );
    }

    #[test]
    fn requirement_page_carries_the_mockup_structure() {
        let html = render_requirement("default", &requirement_fixture());
        assert!(html.contains("class=\"accent-bar requirement\""));
        assert!(html.contains("<h1>SaveInvoice shall split each claim item into portions</h1>"));
        assert!(html.contains("type-badge requirement"));
        assert!(html.contains("status-badge discovery"));
        assert!(html.contains("<span class=\"id-chip\">req_saveinvoice_split</span>"));
        assert!(html.contains("Statement"));
        assert!(html.contains("Resolving Decision"));
        assert!(html.contains("blockquote class=\"position\""));
        assert!(html.contains("Downstream Territory"));
        assert!(html.contains("Field Notes"));
    }

    #[test]
    fn requirement_page_links_every_code_reference() {
        let html = render_requirement("default", &requirement_fixture());
        // Rule evidence links to the host blob URL with line anchors.
        assert!(html
            .contains("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156"));
        // Source citation reference pins to the source commit.
        assert!(html.contains("https://github.com/exampleorg/ex-api/blob/abc1234/docs/award.md"));
        // Field-note bodies linkify code refs and test-case names.
        assert!(html
            .contains("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L211-L233"));
        assert!(html.contains(">testCreateGapInvoiceOnly</a>"));
    }

    #[test]
    fn requirement_page_numbers_source_citations_in_the_margin() {
        let html = render_requirement("default", &requirement_fixture());
        assert!(html.contains("<span class=\"cite-num\">[1]</span>"));
        assert!(html.contains("<a href=\"/sources/source_schads/\">SCHADS Award mapping</a>"));
        assert!(html.contains("clause 10.3"));
    }

    #[test]
    fn requirement_page_shows_lineage_with_the_current_entry_unlinked() {
        let html = render_requirement("default", &requirement_fixture());
        assert!(html.contains("<a href=\"/requirements/req_platform/\">ExampleOrg platform</a>"));
        assert!(html.contains("<li class=\"current\">SaveInvoice shall split each claim item"));
    }

    #[test]
    fn requirement_page_attributes_borrowed_threads_to_their_parent() {
        let html = render_requirement("default", &requirement_fixture());
        assert!(html.contains("thr_resolution_res_split_0"));
        assert!(html.contains("on resolution res_split"));
        assert!(html.contains("1 message · active"));
        assert!(html.contains(">Agent</span>"));
    }

    #[test]
    fn field_notes_who_shows_a_readable_role_not_the_raw_message_id() {
        let html = render_requirement("default", &requirement_fixture());
        assert!(
            !html.contains("msg_000001"),
            "the internal message id should never be shown as if it were an author name"
        );
        assert!(html.contains("<span class=\"who\">Agent</span>"));
    }

    #[test]
    fn gaps_render_as_dashed_citations_and_are_never_suppressed() {
        let html = render_requirement("default", &gappy_requirement_fixture());
        assert_eq!(html.matches("citation gap").count(), 3);
        assert!(html.contains("source ref points at source_missing"));
        assert!(html.contains("no source refs recorded on this requirement"));
        assert!(html.contains("resolved with no resolving decision"));
    }

    #[test]
    fn gappy_page_keeps_the_fog_visible() {
        let html = render_requirement("default", &gappy_requirement_fixture());
        assert!(html.contains("Which award clauses apply is still unclear."));
    }

    #[test]
    fn resolution_page_renders_inputs_as_citations_and_attribution() {
        let html = render_resolution("default", &resolution_fixture());
        assert!(html.contains("class=\"accent-bar resolution\""));
        assert!(html.contains("status-badge approved"));
        assert!(html.contains("<span class=\"cite-num\">[1]</span>"));
        assert!(html.contains("<span class=\"cite-type\">Technical</span>"));
        assert!(
            html.contains("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L59-L69")
        );
        assert!(html.contains("Ben Nasraoui"));
        assert!(html.contains("18 Apr 2026"));
        assert!(html.contains("97%"));
        assert!(html.contains(
            "<a href=\"/requirements/req_saveinvoice_split/\">SaveInvoice shall split each claim item into portions</a>"
        ));
    }

    #[test]
    fn rule_page_links_evidence_but_leaves_prose_references_as_text() {
        let html = render_rule("default", &rule_fixture());
        assert!(html.contains("class=\"accent-bar rule\""));
        assert!(html.contains("SAH-INV-016"));
        assert!(html
            .contains("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156"));
        assert!(html.contains("SCHADS Award clause 10.3"));
        assert!(!html.contains("<a>SCHADS Award clause 10.3</a>"));
        assert!(!html.contains("href=\"\""));
        assert!(html.contains("sev high"));
        assert!(html.contains("prohibition"));
    }

    #[test]
    fn source_page_shows_the_commit_pin_and_referenced_requirements() {
        let html = render_source("default", &source_fixture());
        assert!(html.contains("class=\"accent-bar source\""));
        assert!(html.contains("abc1234"));
        assert!(html.contains("https://example.test/award"));
        assert!(html.contains("4 May 2024"));
        assert!(html.contains(
            "<a href=\"/requirements/req_saveinvoice_split/\">SaveInvoice shall split each claim item into portions</a>"
        ));
    }

    #[test]
    fn index_page_lists_roots_counts_orphans_and_gaps() {
        let html = render_index("default", &index_fixture());
        assert!(html.contains("Provenance atlas — default"));
        assert!(html.contains("<a class=\"entry-title\" href=\"/requirements/req_platform/\">"));
        assert!(html.contains("2 refinements · 1 decision · 1 rule"));
        assert!(html.contains("Orphaned Records"));
        assert!(html.contains("<a href=\"/rules/rule_orphan/\">ORPH-001</a>"));
        assert!(html.contains("<a href=\"/sources/source_unused/\">Unused API spec</a>"));
        assert!(html.contains("citation gap"));
        assert!(html.contains("source_unused is referenced by nothing"));
    }

    #[test]
    fn index_page_on_a_truly_empty_scope_shows_the_honest_empty_state() {
        let page = ScopeIndexPage {
            id: PageId::new(PageKind::ScopeIndex, "default"),
            scope: "default".to_string(),
            title: "Provenance atlas — default".to_string(),
            counts: CorpusCounts::default(),
            roots: vec![],
            gaps: vec![],
            orphans: OrphanReport {
                rules: vec![],
                resolutions: vec![],
                sources: vec![],
            },
        };
        let html = render_index("default", &page);
        assert!(html.contains("No requirements recorded in this scope."));
        assert!(!html.contains("Orphaned Records"));
        assert!(!html.contains("class=\"margin-head\">Gaps"));
        assert!(!html.contains("citation gap"));
    }

    #[test]
    fn not_found_page_names_the_missing_path() {
        let html = render_not_found("default", "/rules/missing/");
        assert!(html.contains("Page not found"));
        assert!(html.contains("/rules/missing/"));
    }

    #[test]
    fn snapshot_requirement_page_with_rules_and_thread() {
        insta::assert_snapshot!(render_requirement("default", &requirement_fixture()));
    }

    #[test]
    fn snapshot_requirement_page_with_gaps() {
        insta::assert_snapshot!(render_requirement("default", &gappy_requirement_fixture()));
    }

    #[test]
    fn snapshot_scope_index_page() {
        insta::assert_snapshot!(render_index("default", &index_fixture()));
    }

    #[test]
    fn rendered_text_is_html_escaped() {
        let mut page = gappy_requirement_fixture();
        page.title = "Overtime > 38h & \"loading\" <rules>".to_string();
        let html = render_requirement("default", &page);
        assert!(
            html.contains("Overtime &gt; 38h &amp; &quot;loading&quot; &lt;rules&gt;")
                || html.contains("Overtime &gt; 38h &amp; \"loading\" &lt;rules&gt;")
        );
        assert!(!html.contains("<rules>"));
    }
}
