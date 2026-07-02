use std::path::Path;

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use crate::parser::contains_annotation_marker;
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

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FileScan {
    pub file_path: Utf8PathBuf,
    pub language: Language,
    pub annotations: Vec<AnnotationLocation>,
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
    let mut warnings = Vec::new();
    let lines = content.lines().collect::<Vec<_>>();
    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
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
        warnings,
    }
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
