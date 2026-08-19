use provenance_macros::rule;

use crate::{protected_spans::ProtectedSpans, sentence::Sentence};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Count {
    Exact(usize),
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnitKind {
    Number,
    CapitalizedWord,
    SemanticCandidate,
    Other,
}

#[derive(Clone, Copy, Debug)]
struct Unit {
    start: usize,
    end: usize,
    kind: UnitKind,
}

/// Counts balanced parenthetical text once in its containing sentence.
#[rule("rule_ste100_parenthetical_counting")]
pub fn count(text: &str, sentence: Sentence, protected: &ProtectedSpans) -> Count {
    if sentence.indeterminate || protected.is_indeterminate() {
        return Count::Indeterminate;
    }

    let mut offset = sentence.start;
    let mut units = Vec::new();
    let mut unclear_punctuation = false;
    while offset < sentence.end {
        let character = next_char(text, offset);
        let unit = match character {
            '"' | '“' => quoted_unit(protected, offset),
            '(' => parenthetical_unit(text, offset),
            character if character.is_ascii_digit() => Some(number_unit(text, offset)),
            character if character.is_alphabetic() => Some(hyphenated_unit(text, offset)),
            _ => None,
        };

        if let Some(unit) = unit {
            offset = unit.end;
            units.push(unit);
        } else {
            unclear_punctuation |= matches!(
                character,
                '\'' | '‘' | '’' | '′' | '″' | '‵' | '‶' | '/' | '\\'
            ) || is_unresolved_grouping_mark(text, offset, character);
            offset += character.len_utf8();
        }
    }

    if unclear_punctuation || has_rule_8_6_meaning_choice(text, &units) {
        Count::Indeterminate
    } else {
        Count::Exact(units.len())
    }
}

/// Consumes a mechanically delimited quotation as one count unit.
#[rule("rule_ste100_explicit_quotation_counting")]
fn quoted_unit(protected: &ProtectedSpans, start: usize) -> Option<Unit> {
    protected.quotation_at(start).map(|span| Unit {
        start: span.start,
        end: span.end,
        kind: UnitKind::Other,
    })
}

fn parenthetical_unit(text: &str, start: usize) -> Option<Unit> {
    let relative_end = text[start + 1..].find(')')?;
    Some(Unit {
        start,
        end: start + relative_end + 2,
        kind: UnitKind::Other,
    })
}

/// Consumes one digit-form number when no semantic grouping is applied.
#[rule("rule_ste100_number_counting")]
fn number_unit(text: &str, start: usize) -> Unit {
    let mut end = start;
    let mut decimal_seen = false;
    while end < text.len() {
        let character = next_char(text, end);
        if character.is_ascii_digit() {
            end += 1;
        } else if character == '.'
            && !decimal_seen
            && text[end + 1..]
                .chars()
                .next()
                .is_some_and(|next| next.is_ascii_digit())
        {
            decimal_seen = true;
            end += 1;
        } else {
            break;
        }
    }
    let kind = if end < text.len() && next_char(text, end).is_alphabetic() {
        end = consume_alphanumeric(text, end);
        UnitKind::SemanticCandidate
    } else {
        UnitKind::Number
    };
    Unit { start, end, kind }
}

/// Consumes a word or an unspaced hyphenated group as one count unit.
#[rule("rule_ste100_hyphenated_group_counting")]
fn hyphenated_unit(text: &str, start: usize) -> Unit {
    let first = next_char(text, start);
    let mut end = consume_alphanumeric(text, start);
    while text[end..].starts_with('-') {
        let after_hyphen = end + 1;
        if after_hyphen >= text.len() || !next_char(text, after_hyphen).is_alphanumeric() {
            break;
        }
        end = consume_alphanumeric(text, after_hyphen);
    }
    Unit {
        start,
        end,
        kind: if first.is_uppercase() {
            UnitKind::CapitalizedWord
        } else {
            UnitKind::Other
        },
    }
}

/// Suppresses a strict result when Rule 8.6 needs semantic classification.
#[rule("rule_ste100_semantic_count_indeterminate")]
fn has_rule_8_6_meaning_choice(text: &str, units: &[Unit]) -> bool {
    units.iter().enumerate().any(|(index, unit)| {
        unit.kind == UnitKind::SemanticCandidate
            || (index > 0 && unit.kind == UnitKind::CapitalizedWord)
    }) || units.windows(2).any(|pair| {
        let between = &text[pair[0].end..pair[1].start];
        let separated = !between.is_empty() && between.chars().all(char::is_whitespace);
        separated
            && matches!(
                (pair[0].kind, pair[1].kind),
                (
                    UnitKind::Number,
                    UnitKind::CapitalizedWord | UnitKind::SemanticCandidate | UnitKind::Other
                ) | (UnitKind::CapitalizedWord, UnitKind::CapitalizedWord)
            )
    })
}

fn consume_alphanumeric(text: &str, start: usize) -> usize {
    text[start..]
        .char_indices()
        .take_while(|&(_, character)| character.is_alphanumeric())
        .last()
        .map_or(start, |(offset, character)| {
            start + offset + character.len_utf8()
        })
}

fn is_unresolved_grouping_mark(text: &str, offset: usize, character: char) -> bool {
    matches!(character, '_' | ':' | '-')
        && text[..offset]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric)
        && text[offset + character.len_utf8()..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric)
}

fn next_char(text: &str, offset: usize) -> char {
    text[offset..].chars().next().expect("offset is in text")
}
