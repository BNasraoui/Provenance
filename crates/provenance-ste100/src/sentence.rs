use std::ops::Range;

use provenance_macros::rule;

use crate::protected_spans::ProtectedSpans;

#[derive(Clone, Copy, Debug)]
pub struct Sentence {
    pub start: usize,
    pub end: usize,
    pub indeterminate: bool,
}

/// Keeps ordinary colons within a sentence and leaves list-like input indeterminate.
#[rule("rule_ste100_ordinary_colon_continuity")]
pub fn scan(text: &str, protected: &ProtectedSpans) -> Vec<Sentence> {
    scan_range(text, protected, 0..text.len())
}

pub fn scan_range(text: &str, protected: &ProtectedSpans, range: Range<usize>) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut sentence_start = range.start;
    let mut parentheses = Vec::new();
    let mut top_level_indeterminate = protected.is_indeterminate();

    for (relative_offset, character) in text[range.clone()].char_indices() {
        let offset = range.start + relative_offset;
        if protected.contains_offset(offset) {
            continue;
        }
        match character {
            '(' => {
                if !parentheses.is_empty() {
                    top_level_indeterminate = true;
                }
                parentheses.push(offset);
            }
            ')' => {
                if let Some(open) = parentheses.pop() {
                    let parenthetical = &text[open + 1..offset];
                    push_trimmed(
                        text,
                        open + 1,
                        offset,
                        top_level_indeterminate
                            || !parentheses.is_empty()
                            || has_internal_sentence_punctuation(parenthetical),
                        &mut sentences,
                    );
                } else {
                    top_level_indeterminate = true;
                }
            }
            '.' | '?' | '!'
                if parentheses.is_empty()
                    && (character != '.' || !is_decimal_point(text, offset)) =>
            {
                let end = offset + character.len_utf8();
                let unclear_period = character == '.' && is_unclear_period_boundary(text, end);
                push_trimmed(
                    text,
                    sentence_start,
                    end,
                    top_level_indeterminate || unclear_period,
                    &mut sentences,
                );
                sentence_start = end;
                top_level_indeterminate = protected.is_indeterminate() || unclear_period;
            }
            _ => {}
        }
    }

    top_level_indeterminate |= !parentheses.is_empty();
    push_trimmed(
        text,
        sentence_start,
        range.end,
        top_level_indeterminate,
        &mut sentences,
    );
    sentences
}

fn push_trimmed(
    text: &str,
    start: usize,
    end: usize,
    indeterminate: bool,
    sentences: &mut Vec<Sentence>,
) {
    let content = &text[start..end];
    let leading = content.len() - content.trim_start().len();
    let trailing = content.len() - content.trim_end().len();
    if leading + trailing < content.len() {
        let trimmed = &content[leading..content.len() - trailing];
        sentences.push(Sentence {
            start: start + leading,
            end: end - trailing,
            indeterminate: indeterminate || has_list_like_colon(trimmed),
        });
    }
}

fn has_list_like_colon(text: &str) -> bool {
    text.match_indices(':').any(|(colon, _)| {
        text[colon + 1..]
            .chars()
            .take_while(|character| character.is_whitespace())
            .any(|character| character == '\n' || character == '\r')
    })
}

fn has_internal_sentence_punctuation(text: &str) -> bool {
    text.char_indices().any(|(offset, character)| {
        matches!(character, '?' | '!') || (character == '.' && !is_decimal_point(text, offset))
    })
}

fn is_decimal_point(text: &str, offset: usize) -> bool {
    text[..offset]
        .chars()
        .next_back()
        .is_some_and(|character| character.is_ascii_digit())
        && text[offset + 1..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
}

fn is_unclear_period_boundary(text: &str, after_period: usize) -> bool {
    let remainder = &text[after_period..];
    let Some(next) = remainder.chars().next() else {
        return false;
    };
    if !next.is_whitespace() {
        return next.is_alphabetic() || next == '.';
    }

    remainder
        .chars()
        .find(|character| !character.is_whitespace())
        .is_some_and(char::is_lowercase)
}
