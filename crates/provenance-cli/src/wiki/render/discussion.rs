use crate::wiki::links::InlineRef;
use std::fmt::Write as _;

use super::html::{escape_attr, escape_html};

const COLLAPSE_CHARS: usize = 480;
const COLLAPSE_LINES: usize = 10;

pub(super) fn note_html(body: &str, refs: &[InlineRef]) -> String {
    let content = structured_body(body, refs);
    let takeaway = takeaway(body);
    if body.chars().count() > COLLAPSE_CHARS || body.lines().count() > COLLAPSE_LINES {
        if let Some(preview) = takeaway.or_else(|| first_nonempty_line(body)) {
            let preview_class = if takeaway.is_some() {
                "fn-takeaway"
            } else {
                "fn-preview"
            };
            return format!(
                "<details class=\"fn-collapsible\">\n<summary><span class=\"{preview_class}\">{}</span><span class=\"fn-expand\">Expand note</span></summary>\n<div class=\"fn-content\">\n{content}</div>\n</details>\n",
                escape_html(preview)
            );
        }
    }

    let mut html = String::new();
    if let Some(takeaway) = takeaway {
        writeln!(
            html,
            "<p class=\"fn-takeaway\">{}</p>",
            escape_html(takeaway)
        )
        .expect("writing to a String should not fail");
    }
    write!(html, "<div class=\"fn-content\">\n{content}</div>\n")
        .expect("writing to a String should not fail");
    html
}

fn takeaway(body: &str) -> Option<&str> {
    let first = first_nonempty_line(body)?;
    if !is_structural_line(first) {
        if let Some(end) = sentence_end(first) {
            return Some(&first[..end]);
        }
        if leading_line_stands_alone(body) {
            return Some(first);
        }
    }

    let final_line = body
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    let lower = final_line.to_ascii_lowercase();
    ["conclusion:", "takeaway:", "summary:", "result:"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
        .then_some(final_line)
}

fn leading_line_stands_alone(body: &str) -> bool {
    let mut lines = body.lines().skip_while(|line| line.trim().is_empty());
    let _first = lines.next();
    lines.next().is_some_and(|line| line.trim().is_empty())
}

fn first_nonempty_line(body: &str) -> Option<&str> {
    body.lines().map(str::trim).find(|line| !line.is_empty())
}

fn sentence_end(line: &str) -> Option<usize> {
    for (index, character) in line.char_indices() {
        if matches!(character, '.' | '?' | '!') {
            let end = index + character.len_utf8();
            if end == line.len() || line[end..].starts_with(char::is_whitespace) {
                return Some(end);
            }
        }
    }
    None
}

fn is_structural_line(line: &str) -> bool {
    line.ends_with(':')
        || line.starts_with("```")
        || ordered_item(line).is_some()
        || unordered_item(line).is_some()
        || serde_json::from_str::<serde_json::Value>(line).is_ok()
}

fn structured_body(body: &str, refs: &[InlineRef]) -> String {
    let lines = line_ranges(body);
    let mut html = String::new();
    let mut index = 0;
    while index < lines.len() {
        let line = &body[lines[index].0..lines[index].1];
        if line.trim().is_empty() {
            index += 1;
        } else if line.trim_start().starts_with("```") {
            index = render_fence(body, &lines, index, &mut html);
        } else if ordered_item(line).is_some() {
            index = render_list(body, refs, &lines, index, true, &mut html);
        } else if unordered_item(line).is_some() {
            index = render_list(body, refs, &lines, index, false, &mut html);
        } else if let Some(end) = json_block_end(body, &lines, index) {
            render_code(&body[lines[index].0..lines[end - 1].1], None, &mut html);
            index = end;
        } else {
            index = render_paragraph(body, refs, &lines, index, &mut html);
        }
    }
    html
}

fn line_ranges(body: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for line in body.split_inclusive('\n') {
        let end = start + line.trim_end_matches('\n').len();
        ranges.push((start, end));
        start += line.len();
    }
    if ranges.is_empty() {
        ranges.push((0, 0));
    }
    ranges
}

fn render_fence(body: &str, lines: &[(usize, usize)], start: usize, html: &mut String) -> usize {
    let opener = body[lines[start].0..lines[start].1].trim();
    let language = opener
        .strip_prefix("```")
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let mut end = start + 1;
    while end < lines.len()
        && !body[lines[end].0..lines[end].1]
            .trim_start()
            .starts_with("```")
    {
        end += 1;
    }
    let content_start = lines.get(start + 1).map_or(lines[start].1, |line| line.0);
    let content_end = lines.get(end).map_or(body.len(), |line| line.0);
    render_code(
        body[content_start..content_end].trim_end_matches('\n'),
        language,
        html,
    );
    (end + 1).min(lines.len())
}

fn render_code(content: &str, language: Option<&str>, html: &mut String) {
    let class = language.map_or_else(String::new, |language| {
        format!(" class=\"language-{}\"", escape_attr(language))
    });
    writeln!(
        html,
        "<pre class=\"fn-code\"><code{class}>{}</code></pre>",
        escape_html(content.trim())
    )
    .expect("writing to a String should not fail");
}

fn render_list(
    body: &str,
    refs: &[InlineRef],
    lines: &[(usize, usize)],
    start: usize,
    ordered: bool,
    html: &mut String,
) -> usize {
    let tag = if ordered { "ol" } else { "ul" };
    writeln!(html, "<{tag} class=\"fn-list-block\">").expect("writing to a String should not fail");
    let mut index = start;
    let mut expected_number: u64 = 1;
    while index < lines.len() {
        let line = &body[lines[index].0..lines[index].1];
        let item = if ordered {
            ordered_item(line)
        } else {
            unordered_item(line)
        };
        let Some(content_offset) = item else { break };
        // Keep the authored number when it breaks sequence, so "1., 3., 7."
        // stays 1., 3., 7. instead of being silently renumbered by the
        // browser's default <ol> counting.
        let mut value_attr = String::new();
        if ordered {
            if let Some(number) = ordered_number(line) {
                if number != expected_number {
                    write!(value_attr, " value=\"{number}\"")
                        .expect("writing to a String should not fail");
                }
                expected_number = number;
            }
        }
        expected_number = expected_number.saturating_add(1);
        let content_start = lines[index].0 + content_offset;
        writeln!(
            html,
            "<li{value_attr}>{}</li>",
            linkify_range(body, refs, content_start, lines[index].1)
        )
        .expect("writing to a String should not fail");
        index += 1;
    }
    writeln!(html, "</{tag}>").expect("writing to a String should not fail");
    index
}

fn ordered_number(line: &str) -> Option<u64> {
    let text = line.trim_start();
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();
    text[..digits].parse().ok()
}

fn ordered_item(line: &str) -> Option<usize> {
    let indent = line.len() - line.trim_start().len();
    let text = &line[indent..];
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let rest = &text[digits..];
    let marker = rest.chars().next()?;
    if !matches!(marker, '.' | ')') {
        return None;
    }
    let after_marker = &rest[marker.len_utf8()..];
    let whitespace = after_marker.len() - after_marker.trim_start().len();
    (whitespace > 0).then_some(indent + digits + marker.len_utf8() + whitespace)
}

fn unordered_item(line: &str) -> Option<usize> {
    let indent = line.len() - line.trim_start().len();
    let text = &line[indent..];
    ["- ", "* ", "+ "]
        .iter()
        .find_map(|marker| text.starts_with(marker).then_some(indent + marker.len()))
}

fn json_block_end(body: &str, lines: &[(usize, usize)], start: usize) -> Option<usize> {
    let first = body[lines[start].0..lines[start].1].trim_start();
    if !first.starts_with(['{', '[']) {
        return None;
    }
    let mut end = start + 1;
    while end < lines.len() && !body[lines[end].0..lines[end].1].trim().is_empty() {
        end += 1;
    }
    let candidate = body[lines[start].0..lines[end - 1].1].trim();
    serde_json::from_str::<serde_json::Value>(candidate)
        .is_ok()
        .then_some(end)
}

fn render_paragraph(
    body: &str,
    refs: &[InlineRef],
    lines: &[(usize, usize)],
    start: usize,
    html: &mut String,
) -> usize {
    let mut end = start + 1;
    while end < lines.len() {
        let line = &body[lines[end].0..lines[end].1];
        if line.trim().is_empty()
            || line.trim_start().starts_with("```")
            || ordered_item(line).is_some()
            || unordered_item(line).is_some()
            || json_block_end(body, lines, end).is_some()
        {
            break;
        }
        end += 1;
    }
    let range_end = lines[end - 1].1;
    writeln!(
        html,
        "<p>{}</p>",
        linkify_range(body, refs, lines[start].0, range_end)
    )
    .expect("writing to a String should not fail");
    end
}

fn linkify_range(body: &str, refs: &[InlineRef], start: usize, end: usize) -> String {
    let mut html = String::new();
    let mut cursor = start;
    for inline in refs {
        if inline.start < cursor || inline.start < start || inline.end > end {
            continue;
        }
        html.push_str(&escape_html(&body[cursor..inline.start]));
        if let Some(href) = &inline.href {
            write!(
                html,
                "<a class=\"src\" href=\"{}\">{}</a>",
                escape_attr(href),
                escape_html(&inline.label)
            )
            .expect("writing to a String should not fail");
        } else {
            html.push_str(&escape_html(&inline.label));
        }
        cursor = inline.end;
    }
    html.push_str(&escape_html(&body[cursor..end]));
    html
}
