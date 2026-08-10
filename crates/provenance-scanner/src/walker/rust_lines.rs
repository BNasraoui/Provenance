//! Per-line lexical states for Rust source.
//!
//! `rust_line_states` records, for each line, whether the line starts in
//! code, a string, a raw string, or a (possibly nested) block comment. The
//! walker gates both channels on these states so directives and attributes
//! inside strings never bind.

use super::Language;
use crate::parser::annotation_marker_positions;
use crate::string_context::obvious_string_is_open;

pub(super) fn rust_line_states(language: Language, lines: &[&str]) -> Vec<RustLexicalState> {
    let mut state = RustLexicalState::Code;
    lines
        .iter()
        .map(|line| {
            let line_state = state;
            if language == Language::Rust {
                advance_rust_state(line, &mut state);
            }
            line_state
        })
        .collect()
}

pub(super) fn rust_annotation_marker_position(
    line: &str,
    initial_state: RustLexicalState,
) -> Option<usize> {
    annotation_marker_positions(line)
        .filter(|position| {
            let mut state = initial_state;
            advance_rust_state(&line[..*position], &mut state);
            let closed_multiline_string = matches!(
                initial_state,
                RustLexicalState::Quoted | RustLexicalState::Raw(_)
            ) && state == RustLexicalState::Code;
            !matches!(state, RustLexicalState::Quoted | RustLexicalState::Raw(_))
                && (closed_multiline_string || !obvious_string_is_open(&line[..*position]))
        })
        .min()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RustLexicalState {
    Code,
    Quoted,
    Raw(usize),
    BlockComment(usize),
}

fn advance_rust_state(line: &str, state: &mut RustLexicalState) {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match *state {
            RustLexicalState::Code => match bytes[index] {
                b'/' if bytes.get(index + 1) == Some(&b'/') => break,
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    *state = RustLexicalState::BlockComment(1);
                    index += 2;
                }
                b'\'' => {
                    index = character_literal_end(line, index + 1).unwrap_or(index) + 1;
                }
                b'"' => {
                    *state = RustLexicalState::Quoted;
                    index += 1;
                }
                b'r' => {
                    let hashes = bytes[index + 1..]
                        .iter()
                        .take_while(|byte| **byte == b'#')
                        .count();
                    if bytes.get(index + hashes + 1) == Some(&b'"') {
                        *state = RustLexicalState::Raw(hashes);
                        index += hashes + 2;
                    } else {
                        index += 1;
                    }
                }
                _ => index += 1,
            },
            RustLexicalState::Quoted => match bytes[index] {
                b'\\' => index = (index + 2).min(bytes.len()),
                b'"' => {
                    *state = RustLexicalState::Code;
                    index += 1;
                }
                _ => index += 1,
            },
            RustLexicalState::Raw(hashes) => {
                let closes = bytes[index] == b'"'
                    && bytes[index + 1..]
                        .iter()
                        .take(hashes)
                        .all(|byte| *byte == b'#')
                    && bytes.len().saturating_sub(index + 1) >= hashes;
                if closes {
                    *state = RustLexicalState::Code;
                    index += hashes + 1;
                } else {
                    index += 1;
                }
            }
            RustLexicalState::BlockComment(depth) => {
                if bytes[index..].starts_with(b"/*") {
                    *state = RustLexicalState::BlockComment(depth + 1);
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    *state = if depth == 1 {
                        RustLexicalState::Code
                    } else {
                        RustLexicalState::BlockComment(depth - 1)
                    };
                    index += 2;
                } else {
                    index += 1;
                }
            }
        }
    }
}

fn character_literal_end(line: &str, index: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let content_end = if bytes.get(index) == Some(&b'\\') {
        match bytes.get(index + 1) {
            Some(b'x') => index + 4,
            Some(b'u') if bytes.get(index + 2) == Some(&b'{') => {
                index + 3 + bytes[index + 3..].iter().position(|byte| *byte == b'}')? + 1
            }
            Some(_) => index + 2,
            None => return None,
        }
    } else {
        index + line.get(index..)?.chars().next()?.len_utf8()
    };
    (bytes.get(content_end) == Some(&b'\'')).then_some(content_end)
}
