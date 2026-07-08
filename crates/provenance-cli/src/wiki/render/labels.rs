use crate::wiki::model::PageKind;
use provenance_core::{
    NodeType, RequirementStatus, ResolutionInputType, ResolutionStatus, RuleModality, RuleSeverity,
    RuleStatus, RuleType, SourceType, ThreadStatus,
};

use super::html::icon_svg;

pub(in crate::wiki::render) fn status_badge(word: &str) -> String {
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

pub(in crate::wiki::render) fn sev_chip(class_word: &str, label: &str) -> String {
    format!("<span class=\"sev {class_word}\">{label}</span>")
}

pub(in crate::wiki::render) fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

pub(in crate::wiki::render) fn counted(count: usize, singular: &str, plural: &str) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
}

pub(in crate::wiki::render) const fn kind_class(kind: PageKind) -> &'static str {
    match kind {
        PageKind::ScopeIndex => "scope-index",
        PageKind::Requirement => "requirement",
        PageKind::Resolution => "resolution",
        PageKind::Rule => "rule",
        PageKind::Source => "source",
    }
}

pub(in crate::wiki::render) const fn kind_label(kind: PageKind) -> &'static str {
    match kind {
        PageKind::ScopeIndex => "Scope",
        PageKind::Requirement => "Requirement",
        PageKind::Resolution => "Resolution",
        PageKind::Rule => "Rule",
        PageKind::Source => "Source",
    }
}

pub(in crate::wiki::render) const fn kind_icon(kind: PageKind) -> &'static str {
    match kind {
        PageKind::ScopeIndex | PageKind::Rule | PageKind::Source => "i-book-open",
        PageKind::Requirement => "i-git-branch",
        PageKind::Resolution => "i-scale",
    }
}

pub(in crate::wiki::render) const fn requirement_status_word(
    status: &RequirementStatus,
) -> &'static str {
    match status {
        RequirementStatus::Active => "active",
        RequirementStatus::Discovery => "discovery",
        RequirementStatus::Refinement => "refinement",
        RequirementStatus::Resolved => "resolved",
    }
}

pub(in crate::wiki::render) const fn resolution_status_word(
    status: &ResolutionStatus,
) -> &'static str {
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

pub(in crate::wiki::render) const fn rule_status_word(status: &RuleStatus) -> &'static str {
    match status {
        RuleStatus::Draft => "draft",
        RuleStatus::Review => "review",
        RuleStatus::Active => "active",
        RuleStatus::Deprecated => "deprecated",
        RuleStatus::Archived => "archived",
    }
}

pub(in crate::wiki::render) const fn thread_status_word(status: &ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Active => "active",
        ThreadStatus::Resolved => "resolved",
        ThreadStatus::Archived => "archived",
    }
}

pub(in crate::wiki::render) const fn severity_word(severity: &RuleSeverity) -> &'static str {
    match severity {
        RuleSeverity::Low => "low",
        RuleSeverity::Medium => "medium",
        RuleSeverity::High => "high",
        RuleSeverity::Critical => "critical",
    }
}

pub(in crate::wiki::render) const fn modality_word(modality: &RuleModality) -> &'static str {
    match modality {
        RuleModality::Obligation => "obligation",
        RuleModality::Prohibition => "prohibition",
        RuleModality::Necessity => "necessity",
    }
}

pub(in crate::wiki::render) const fn rule_type_word(rule_type: &RuleType) -> &'static str {
    match rule_type {
        RuleType::Business => "business",
        RuleType::Functional => "functional",
        RuleType::Technical => "technical",
    }
}

pub(in crate::wiki::render) const fn source_type_label(source_type: &SourceType) -> &'static str {
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

pub(in crate::wiki::render) const fn input_type_label(
    input_type: &ResolutionInputType,
) -> &'static str {
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

pub(in crate::wiki::render) const fn node_type_word(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Source => "source",
        NodeType::Requirement => "requirement",
        NodeType::Resolution => "resolution",
        NodeType::Rule => "rule",
        NodeType::Topic => "topic",
        NodeType::Question => "question",
    }
}

pub(in crate::wiki::render) fn format_date_iso_ms(ms: i64) -> String {
    let (year, month, day) = civil_from_days(ms.div_euclid(86_400_000));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Formats an epoch-milliseconds timestamp as a civil UTC date, mockup
/// style: `18 Apr 2026`.
pub(in crate::wiki::render) fn format_date_ms(ms: i64) -> String {
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
pub(in crate::wiki::render) fn format_confidence(confidence: f64) -> String {
    format!("{}%", (confidence * 100.0).round() as u32)
}
