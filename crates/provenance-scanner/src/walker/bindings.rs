//! Recognizes rule and verification binding sites, one line at a time.
//!
//! Rust binds through `#[rule]`/`#[verifies]` attributes, Python through
//! `@rule(...)` decorators, and JS/TS/Go/Java through `rule(...)` and
//! `verifies(...)` calls found by the binding lexer.

use std::str::FromStr;

use super::Language;
use crate::binding_lexer::call_arguments;
use crate::parser::Verification;

pub(super) fn parse_binding_line(
    language: Language,
    line: &str,
    in_block_comment: bool,
) -> Option<(String, Option<Verification>)> {
    match language {
        Language::Rust => (!in_block_comment)
            .then(|| parse_attribute_line(line))
            .flatten(),
        Language::Python => parse_python_decorator(line),
        Language::JavaScript | Language::TypeScript => parse_script_call(line, in_block_comment),
        Language::Go | Language::Java => parse_rule_call(line, in_block_comment),
    }
}

fn parse_python_decorator(line: &str) -> Option<(String, Option<Verification>)> {
    let trimmed = line.trim_start();
    let decorator = trimmed.strip_prefix('@')?;
    let rest = decorator.strip_prefix("rule(").or_else(|| {
        decorator
            .split_once(".rule(")
            .filter(|(qualifier, _)| !qualifier.is_empty())
            .map(|(_, rest)| rest)
    })?;
    Some((quoted_literal(rest)?.0, None))
}

fn parse_script_call(line: &str, in_block_comment: bool) -> Option<(String, Option<Verification>)> {
    if let Some(rest) = call_arguments(line, in_block_comment, "verifies") {
        let (rule_id, after_id) = quoted_literal(rest)?;
        let method = argument_after_comma(after_id)?;
        return Some((rule_id, Some(Verification::from_str(method).ok()?)));
    }
    parse_rule_call(line, in_block_comment)
}

fn parse_rule_call(line: &str, in_block_comment: bool) -> Option<(String, Option<Verification>)> {
    let rest = call_arguments(line, in_block_comment, "rule")?;
    let (rule_id, after_id) = quoted_literal(rest)?;
    after_id
        .trim_start()
        .starts_with(',')
        .then_some((rule_id, None))
}

fn quoted_literal(rest: &str) -> Option<(String, &str)> {
    let rest = rest.trim_start();
    let quote = rest.chars().next()?;
    if !matches!(quote, '\'' | '"' | '`') {
        return None;
    }
    let after_quote = &rest[quote.len_utf8()..];
    let end = after_quote.find(quote)?;
    let literal = &after_quote[..end];
    if literal.is_empty() {
        return None;
    }
    Some((literal.to_string(), &after_quote[end + quote.len_utf8()..]))
}

fn argument_after_comma(rest: &str) -> Option<&str> {
    let argument = rest.trim_start().strip_prefix(',')?.trim_start();
    let unquoted = argument.strip_prefix(['\'', '"', '`']).unwrap_or(argument);
    let method = unquoted
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()?;
    (!method.is_empty()).then_some(method)
}

/// Recognizes single-line `#[rule("id")]` and `#[verifies("id", method)]`
/// attributes. The proc macros reject malformed arguments at compile time, so
/// anything found in compiling code is well-formed; lines that do not match
/// are silently skipped.
fn parse_attribute_line(line: &str) -> Option<(String, Option<Verification>)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("#[rule(") {
        return Some((string_literal(rest)?, None));
    }
    let rest = trimmed.strip_prefix("#[verifies(")?;
    let rule_id = string_literal(rest)?;
    let after_literal = rest.split_once(',')?.1;
    let method = after_literal.trim().trim_end_matches(")]").trim();
    Some((rule_id, Some(Verification::from_str(method).ok()?)))
}

fn string_literal(rest: &str) -> Option<String> {
    let after_quote = rest.strip_prefix('"')?;
    let (literal, _) = after_quote.split_once('"')?;
    (!literal.is_empty()).then(|| literal.to_string())
}
