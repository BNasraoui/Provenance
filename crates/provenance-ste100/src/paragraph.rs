use std::ops::Range;

use crate::protected_spans::ProtectedSpans;

pub fn scan(text: &str, protected: &ProtectedSpans) -> Option<Vec<Range<usize>>> {
    if protected.is_indeterminate() || has_unpaired_carriage_return(text, protected) {
        return None;
    }

    let mut paragraphs = Vec::new();
    let mut paragraph_start = None;
    let mut paragraph_end = 0;
    let mut line_start = 0;

    for newline in text.match_indices('\n').map(|(offset, _)| offset) {
        let content_end = newline
            .checked_sub(1)
            .filter(|offset| text.as_bytes()[*offset] == b'\r')
            .unwrap_or(newline);
        consume_line(
            text,
            line_start,
            content_end,
            protected.contains_offset(newline),
            &mut paragraph_start,
            &mut paragraph_end,
            &mut paragraphs,
        );
        line_start = newline + 1;
    }

    consume_line(
        text,
        line_start,
        text.len(),
        false,
        &mut paragraph_start,
        &mut paragraph_end,
        &mut paragraphs,
    );
    close_paragraph(&mut paragraph_start, paragraph_end, &mut paragraphs);
    Some(paragraphs)
}

fn consume_line(
    text: &str,
    start: usize,
    end: usize,
    protected_newline: bool,
    paragraph_start: &mut Option<usize>,
    paragraph_end: &mut usize,
    paragraphs: &mut Vec<Range<usize>>,
) {
    let line = &text[start..end];
    if line.trim().is_empty() && !protected_newline {
        close_paragraph(paragraph_start, *paragraph_end, paragraphs);
        return;
    }

    if let Some((trimmed_start, trimmed_end)) = trimmed_range(text, start, end) {
        paragraph_start.get_or_insert(trimmed_start);
        *paragraph_end = trimmed_end;
    }
}

fn close_paragraph(start: &mut Option<usize>, end: usize, paragraphs: &mut Vec<Range<usize>>) {
    if let Some(start) = start.take() {
        paragraphs.push(start..end);
    }
}

fn trimmed_range(text: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let content = &text[start..end];
    let leading = content.len() - content.trim_start().len();
    let trailing = content.len() - content.trim_end().len();
    (leading + trailing < content.len()).then_some((start + leading, end - trailing))
}

fn has_unpaired_carriage_return(text: &str, protected: &ProtectedSpans) -> bool {
    text.match_indices('\r').any(|(offset, _)| {
        !protected.contains_offset(offset) && text.as_bytes().get(offset + 1) != Some(&b'\n')
    })
}
