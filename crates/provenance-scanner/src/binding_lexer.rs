/// Finds a call outside strings and comments in one source line.
pub fn call_arguments<'a>(
    line: &'a str,
    starts_in_block_comment: bool,
    function: &str,
) -> Option<&'a str> {
    let marker = format!("{function}(");
    scan_line(line, starts_in_block_comment, Some(&marker))
        .0
        .map(|start| &line[start + marker.len()..])
}

/// Carries C-style block-comment state between source lines.
pub fn block_comment_state(line: &str, starts_in_block_comment: bool) -> bool {
    scan_line(line, starts_in_block_comment, None).1
}

#[derive(Clone, Copy)]
pub enum MultilineStyle {
    Backtick,
    TripleBoth,
    TripleDouble,
    RustRaw,
}

pub fn unclosed_multiline_delimiter(line: &str, style: MultilineStyle) -> Option<(String, usize)> {
    if matches!(style, MultilineStyle::RustRaw) {
        return unclosed_rust_raw_delimiter(line);
    }
    let delimiters = match style {
        MultilineStyle::Backtick => &["`"][..],
        MultilineStyle::TripleBoth => &["'''", "\"\"\""][..],
        MultilineStyle::TripleDouble => &["\"\"\""][..],
        MultilineStyle::RustRaw => unreachable!(),
    };
    delimiters
        .iter()
        .filter_map(|delimiter| {
            unpaired_delimiter_start(line, delimiter, matches!(style, MultilineStyle::TripleBoth))
                .map(|start| ((*delimiter).to_string(), start))
        })
        .min_by_key(|(_, start)| *start)
}

pub fn code_outside_multiline_string<'a>(
    line: &'a str,
    style: MultilineStyle,
    active_delimiter: &mut Option<String>,
    starts_in_block_comment: bool,
) -> Option<&'a str> {
    let mut code = line;
    if let Some(delimiter) = active_delimiter.as_deref() {
        let closing = code.find(delimiter)? + delimiter.len();
        code = &code[closing..];
        *active_delimiter = None;
    }
    if starts_in_block_comment {
        return Some(code);
    }
    if let Some((delimiter, opening)) = unclosed_multiline_delimiter(code, style) {
        *active_delimiter = Some(delimiter);
        code = &code[..opening];
    }
    Some(code)
}

fn unclosed_rust_raw_delimiter(line: &str) -> Option<(String, usize)> {
    line.match_indices('r').find_map(|(start, _)| {
        if line[..start].contains("//") || line[..start].contains("/*") {
            return None;
        }
        let after_r = &line[start + 1..];
        let hashes = after_r.bytes().take_while(|byte| *byte == b'#').count();
        let contents = after_r[hashes..].strip_prefix('"')?;
        let closing = format!("\"{}", "#".repeat(hashes));
        (!contents.contains(&closing)).then_some((closing, start))
    })
}

fn unpaired_delimiter_start(
    line: &str,
    delimiter: &str,
    hash_starts_comment: bool,
) -> Option<usize> {
    let bytes = line.as_bytes();
    let marker = delimiter.as_bytes();
    let mut positions = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut idx = 0;
    while idx < bytes.len() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if bytes[idx] == b'\\' {
                escaped = true;
            } else if bytes[idx] == active_quote {
                quote = None;
            }
            idx += 1;
            continue;
        }
        if bytes[idx..].starts_with(b"//") || bytes[idx..].starts_with(b"/*") {
            break;
        }
        if hash_starts_comment && bytes[idx] == b'#' {
            break;
        }
        if bytes[idx..].starts_with(marker) {
            positions.push(idx);
            idx += marker.len();
            continue;
        }
        if matches!(bytes[idx], b'\'' | b'"') {
            quote = Some(bytes[idx]);
        }
        idx += 1;
    }
    (positions.len() % 2 == 1).then(|| positions[positions.len() - 1])
}

fn scan_line(
    line: &str,
    mut in_block_comment: bool,
    marker: Option<&str>,
) -> (Option<usize>, bool) {
    let bytes = line.as_bytes();
    let mut quote = None;
    let mut escaped = false;
    let mut idx = 0;
    while idx < bytes.len() {
        if in_block_comment {
            if bytes[idx..].starts_with(b"*/") {
                in_block_comment = false;
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if bytes[idx] == b'\\' {
                escaped = true;
            } else if bytes[idx] == active_quote {
                quote = None;
            }
            idx += 1;
            continue;
        }
        if bytes[idx..].starts_with(b"//") {
            break;
        }
        if bytes[idx..].starts_with(b"/*") {
            in_block_comment = true;
            idx += 2;
            continue;
        }
        if matches!(bytes[idx], b'\'' | b'"' | b'`') {
            quote = Some(bytes[idx]);
            idx += 1;
            continue;
        }
        if let Some(marker) = marker.filter(|marker| bytes[idx..].starts_with(marker.as_bytes())) {
            let boundary = idx.checked_sub(1).map(|previous| bytes[previous]);
            if !boundary.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
                return (Some(idx), in_block_comment);
            }
            idx += marker.len();
            continue;
        }
        idx += 1;
    }
    (None, in_block_comment)
}
