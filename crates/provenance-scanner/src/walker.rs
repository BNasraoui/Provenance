use std::path::Path;

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use std::str::FromStr;

use crate::parser::{contains_annotation_marker, Verification};
use crate::{parse_annotations, Annotation, ParseWarning};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    Go,
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "jsx" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "java" => Some(Self::Java),
            "go" => Some(Self::Go),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AnnotationLocation {
    pub file_path: Utf8PathBuf,
    pub line: usize,
    pub function_name: Option<String>,
    pub annotation: Annotation,
}

/// A `#[rule]` or `#[verifies]` attribute found in source.
///
/// `verification` is `None` for a `#[rule]` site (the item is the rule) and
/// `Some` for a `#[verifies]` site (the item checks the rule, the method
/// says how).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AttributeBinding {
    pub file_path: Utf8PathBuf,
    pub line: usize,
    pub item_name: Option<String>,
    pub rule_id: String,
    pub verification: Option<Verification>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FileScan {
    pub file_path: Utf8PathBuf,
    pub language: Language,
    pub annotations: Vec<AnnotationLocation>,
    pub bindings: Vec<AttributeBinding>,
    pub warnings: Vec<ParseWarning>,
}

pub fn scan_path(path: &Utf8Path) -> anyhow::Result<Vec<FileScan>> {
    let mut scans = Vec::new();
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(file_path) = Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).ok() else {
            continue;
        };
        let Some(language) = file_path.extension().and_then(Language::from_extension) else {
            continue;
        };
        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("read source file {file_path}"))?;
        scans.push(scan_file(&file_path, language, &content));
    }
    scans.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(scans)
}

pub fn scan_file(file_path: &Utf8Path, language: Language, content: &str) -> FileScan {
    let mut annotations = Vec::new();
    let mut bindings = Vec::new();
    let mut warnings = Vec::new();
    let lines = content.lines().collect::<Vec<_>>();
    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
        if language == Language::Rust {
            if let Some((rule_id, verification)) = parse_attribute_line(line) {
                bindings.push(AttributeBinding {
                    file_path: file_path.to_path_buf(),
                    line: idx + 1,
                    item_name: next_item_name(language, &lines[idx + 1..]),
                    rule_id,
                    verification,
                });
                idx += 1;
                continue;
            }
        }
        if !contains_annotation_marker(line) {
            idx += 1;
            continue;
        }
        let (comment, end_idx) = collect_annotation_comment(&lines, idx);
        let parsed = parse_annotations(&comment);
        warnings.extend(parsed.warnings);
        let function_name = next_function_name(language, &lines[end_idx.saturating_add(1)..]);
        for annotation in parsed.annotations {
            annotations.push(AnnotationLocation {
                file_path: file_path.to_path_buf(),
                line: idx + 1,
                function_name: function_name.clone(),
                annotation,
            });
        }
        idx = end_idx + 1;
    }
    FileScan {
        file_path: file_path.to_path_buf(),
        language,
        annotations,
        bindings,
        warnings,
    }
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

/// Like `next_function_name`, but looks past other attribute lines such as
/// `#[test]`, and also accepts type definitions (`construction` bindings sit
/// on types, not functions).
fn next_item_name(language: Language, following: &[&str]) -> Option<String> {
    following
        .iter()
        .filter(|line| !line.trim_start().starts_with("#["))
        .take(6)
        .find_map(|line| {
            let line = line.trim();
            function_name(language, line).or_else(|| type_name(line))
        })
}

fn type_name(line: &str) -> Option<String> {
    ["struct ", "enum ", "type "]
        .iter()
        .find(|marker| line.contains(*marker))
        .and_then(|marker| token_after(line, marker))
}

fn collect_annotation_comment(lines: &[&str], start: usize) -> (String, usize) {
    let mut end = start;
    while end + 1 < lines.len() && is_comment_continuation(lines[end + 1]) {
        end += 1;
    }
    (lines[start..=end].join("\n"), end)
}

fn is_comment_continuation(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with('*')
        || trimmed.starts_with("/*")
        || trimmed.starts_with("*/")
}

fn next_function_name(language: Language, following: &[&str]) -> Option<String> {
    following
        .iter()
        .take(6)
        .find_map(|line| function_name(language, line.trim()))
}

fn function_name(language: Language, line: &str) -> Option<String> {
    let marker = match language {
        Language::Rust => "fn ",
        Language::Python => "def ",
        Language::Go => "func ",
        Language::JavaScript | Language::TypeScript | Language::Java => " ",
    };
    if matches!(language, Language::JavaScript | Language::TypeScript)
        && line.starts_with("function ")
    {
        return token_after(line, "function ");
    }
    if language == Language::Go && line.starts_with("func (") {
        let after_receiver = line.split_once(") ")?.1;
        return token_after(after_receiver, "");
    }
    token_after(line, marker)
}

fn token_after(line: &str, marker: &str) -> Option<String> {
    let start = line.find(marker)? + marker.len();
    let name = line[start..]
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()?;
    (!name.is_empty()).then(|| name.to_string())
}

#[allow(dead_code)]
const fn _assert_path(_: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_rust_annotation_with_location() {
        let scan = scan_file(
            Utf8Path::new("payroll.rs"),
            Language::Rust,
            "// @provenance rule: SCHADS-PAY-001\nfn pays_overtime() {}",
        );

        assert_eq!(scan.annotations[0].line, 1);
        assert_eq!(
            scan.annotations[0].function_name.as_deref(),
            Some("pays_overtime")
        );
    }

    #[test]
    fn scans_rule_attribute_with_item_name() {
        let scan = scan_file(
            Utf8Path::new("edge_validation.rs"),
            Language::Rust,
            "#[rule(\"rule_prov_edge_endpoint_table\")]\npub fn validate_edge_endpoint() {}",
        );

        assert_eq!(
            scan.bindings,
            vec![AttributeBinding {
                file_path: Utf8Path::new("edge_validation.rs").to_path_buf(),
                line: 1,
                item_name: Some("validate_edge_endpoint".to_string()),
                rule_id: "rule_prov_edge_endpoint_table".to_string(),
                verification: None,
            }]
        );
    }

    #[test]
    fn scans_verifies_attribute_past_test_attribute() {
        let scan = scan_file(
            Utf8Path::new("edge_validation.rs"),
            Language::Rust,
            "#[test]\n#[verifies(\"rule_prov_edge_endpoint_table\", exhaustion)]\nfn endpoint_table_conforms_to_rule_leaf() {}",
        );

        assert_eq!(scan.bindings.len(), 1);
        assert_eq!(
            scan.bindings[0].verification,
            Some(Verification::Exhaustion)
        );
        assert_eq!(
            scan.bindings[0].item_name.as_deref(),
            Some("endpoint_table_conforms_to_rule_leaf")
        );
    }

    #[test]
    fn scans_construction_verifies_attribute_on_a_type() {
        let scan = scan_file(
            Utf8Path::new("tokens.rs"),
            Language::Rust,
            "#[verifies(\"rule_redacted_display\", construction)]\npub struct RedactedToken(String);",
        );

        assert_eq!(
            scan.bindings[0].verification,
            Some(Verification::Construction)
        );
        assert_eq!(scan.bindings[0].item_name.as_deref(), Some("RedactedToken"));
    }

    #[test]
    fn scans_legacy_statesman_annotation_with_location() {
        let scan = scan_file(
            Utf8Path::new("payroll.rs"),
            Language::Rust,
            "// @statesman rule: SCHADS-PAY-001\nfn pays_overtime() {}",
        );

        assert_eq!(scan.annotations.len(), 1);
        assert_eq!(scan.annotations[0].line, 1);
    }
}
