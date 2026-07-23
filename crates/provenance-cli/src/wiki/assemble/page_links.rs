use crate::wiki::model::{PageId, PageLink, RecordKind};
use provenance_core::{Requirement, Resolution, Rule, Source};

pub(super) fn requirement_link(requirement: &Requirement) -> PageLink {
    PageLink {
        target: PageId::new(RecordKind::Requirement, requirement.id.as_str()),
        title: requirement.statement.clone(),
    }
}

pub(super) fn resolution_link(resolution: &Resolution) -> PageLink {
    PageLink {
        target: PageId::new(RecordKind::Resolution, resolution.id.as_str()),
        title: resolution.title.clone(),
    }
}

pub(super) fn rule_link(rule: &Rule) -> PageLink {
    PageLink {
        target: PageId::new(RecordKind::Rule, rule.id.as_str()),
        title: rule_title(rule),
    }
}

pub(super) fn rule_title(rule: &Rule) -> String {
    rule.name.clone().unwrap_or_else(|| rule.rule_code.clone())
}

pub(super) fn source_link(source: &Source) -> PageLink {
    PageLink {
        target: PageId::new(RecordKind::Source, source.id.as_str()),
        title: source.name.clone(),
    }
}
