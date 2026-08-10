use crate::wiki::model::{PageId, PageLink, RecordKind};
use provenance_core::{Requirement, Resolution, Rule, Source};

pub(super) fn requirement_link(requirement: &Requirement) -> PageLink {
    PageLink {
        target: PageId::new(RecordKind::Requirement, requirement.id.as_str()),
        title: reader_title(&requirement.statement),
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
    rule.name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .map_or_else(
            || display_identifier(rule.id.as_str()),
            |name| name.trim().to_string(),
        )
}

pub(super) fn source_link(source: &Source) -> PageLink {
    PageLink {
        target: PageId::new(RecordKind::Source, source.id.as_str()),
        title: source.name.clone(),
    }
}

pub(super) fn display_identifier(identifier: &str) -> String {
    let words = identifier
        .trim()
        .split(['_', '-'])
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut chars = words.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

pub(super) fn reader_title(text: &str) -> String {
    const MAX_CHARS: usize = 70;
    let text = text.trim();
    let clause_end = text
        .char_indices()
        .find_map(|(index, character)| match character {
            ';' | '\n' => Some(index),
            '.' if sentence_period(text, index) => Some(index),
            _ => None,
        });
    let clause = clause_end.map_or(text, |end| &text[..end]).trim();
    if clause.chars().count() <= MAX_CHARS {
        return clause.to_string();
    }

    let byte_limit = clause
        .char_indices()
        .nth(MAX_CHARS - 1)
        .map_or(clause.len(), |(index, _)| index);
    let candidate = &clause[..byte_limit];
    let shortened = candidate
        .rfind(char::is_whitespace)
        .map_or(candidate, |space| &candidate[..space])
        .trim_end();
    format!("{shortened}…")
}

fn sentence_period(text: &str, index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let after = &text[index + 1..];
    if after
        .chars()
        .next()
        .is_some_and(|next| !next.is_whitespace())
    {
        return false;
    }
    let token = text[..index]
        .rsplit_once(char::is_whitespace)
        .map_or(&text[..index], |(_, token)| token);
    !token.contains('.')
}
