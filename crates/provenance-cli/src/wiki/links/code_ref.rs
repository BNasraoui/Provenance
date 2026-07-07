use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LineRange {
    pub start: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u32>,
}

impl LineRange {
    pub const fn new(start: u32, end: Option<u32>) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeRef {
    pub path: String,
    pub lines: Vec<LineRange>,
}

/// Parses a code reference such as `src/UseCase.php:153-156`.
///
/// The path part must look like a file path (no whitespace, and either a
/// directory separator or a dotted file name). Line groups accept single
/// lines, `-`/en-dash ranges, and comma-separated lists.
pub fn parse_code_ref(text: &str) -> Option<CodeRef> {
    let text = text.trim();
    let (path, lines_part) = text
        .split_once(':')
        .map_or((text, None), |(path, lines)| (path, Some(lines)));
    if !is_file_path(path) {
        return None;
    }
    let lines = match lines_part {
        Some(lines_part) => parse_line_ranges(lines_part)?,
        None => Vec::new(),
    };
    Some(CodeRef {
        path: path.to_string(),
        lines,
    })
}

fn is_file_path(path: &str) -> bool {
    if path.is_empty() || path.contains("://") || path.chars().any(char::is_whitespace) {
        return false;
    }
    let file_name = path.rsplit('/').next().unwrap_or(path);
    path.contains('/') || file_name.contains('.')
}

/// Strips a leading "line"/"lines" word (case-insensitive), as in the
/// common human-written form `UseCase.php:lines 153-156`, so the numeric
/// parser underneath never has to know about it.
fn strip_leading_lines_word(part: &str) -> &str {
    let trimmed = part.trim_start();
    for word in ["lines", "line"] {
        if trimmed.len() > word.len()
            && trimmed.as_bytes()[word.len()].is_ascii_whitespace()
            && trimmed[..word.len()].eq_ignore_ascii_case(word)
        {
            return trimmed[word.len()..].trim_start();
        }
    }
    trimmed
}

fn parse_line_ranges(part: &str) -> Option<Vec<LineRange>> {
    let part = strip_leading_lines_word(part);
    part.split(',')
        .map(|group| {
            let group = group.trim();
            let (start, end) = group
                .split_once(['-', '\u{2013}'])
                .map_or((group, None), |(start, end)| {
                    (start.trim(), Some(end.trim()))
                });
            Some(LineRange::new(
                start.parse().ok()?,
                end.map(str::parse).transpose().ok()?,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_code_ref_reads_a_plain_path() {
        let code_ref = parse_code_ref("docs/save-invoice.md").unwrap();
        assert_eq!(code_ref.path, "docs/save-invoice.md");
        assert!(code_ref.lines.is_empty());
    }

    #[test]
    fn parse_code_ref_reads_a_single_line() {
        let code_ref = parse_code_ref("UseCase.php:153").unwrap();
        assert_eq!(code_ref.path, "UseCase.php");
        assert_eq!(code_ref.lines, vec![LineRange::new(153, None)]);
    }

    #[test]
    fn parse_code_ref_reads_a_line_range() {
        let code_ref = parse_code_ref("UseCase.php:153-156").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(153, Some(156))]);
    }

    #[test]
    fn parse_code_ref_accepts_a_leading_lines_word() {
        let code_ref = parse_code_ref("UseCase.php:lines 153-156").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(153, Some(156))]);
    }

    #[test]
    fn parse_code_ref_accepts_a_leading_line_word_singular() {
        let code_ref = parse_code_ref("UseCase.php:line 42").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(42, None)]);
    }

    #[test]
    fn parse_code_ref_accepts_lines_word_case_insensitively_with_extra_space() {
        let code_ref = parse_code_ref("UseCase.php: Lines  59-69").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(59, Some(69))]);
    }

    #[test]
    fn parse_code_ref_accepts_en_dash_ranges() {
        let code_ref = parse_code_ref("UseCase.php:59\u{2013}69").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(59, Some(69))]);
    }

    #[test]
    fn parse_code_ref_reads_comma_separated_line_groups() {
        let code_ref = parse_code_ref("UseCase.php:168, 193, 218").unwrap();
        assert_eq!(
            code_ref.lines,
            vec![
                LineRange::new(168, None),
                LineRange::new(193, None),
                LineRange::new(218, None),
            ]
        );
    }

    #[test]
    fn parse_code_ref_rejects_prose_urls_and_bare_words() {
        assert!(parse_code_ref("Section 7.2 of the award").is_none());
        assert!(parse_code_ref("https://example.com/handbook").is_none());
        assert!(parse_code_ref("README").is_none());
        assert!(parse_code_ref("12:30pm").is_none());
        assert!(parse_code_ref("").is_none());
    }
}
