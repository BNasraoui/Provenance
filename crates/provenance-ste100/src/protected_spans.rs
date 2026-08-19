use provenance_macros::rule;

#[derive(Clone, Debug)]
pub enum ProtectedSpans {
    Exact(Vec<ProtectedSpan>),
    Indeterminate,
}

#[derive(Clone, Copy, Debug)]
pub struct ProtectedSpan {
    pub start: usize,
    pub content_start: usize,
    pub content_end: usize,
    pub end: usize,
}

#[derive(Clone, Copy)]
enum Opening {
    Straight { start: usize },
    Curly { start: usize },
}

/// Identifies exact protected quoted spans or an indeterminate quote structure.
#[rule("rule_ste100_quoted_text_protection")]
pub fn analyze(text: &str) -> ProtectedSpans {
    let mut spans = Vec::new();
    let mut opening = None;

    for (offset, character) in text.char_indices() {
        opening = match (opening, character) {
            (None, '"') => Some(Opening::Straight { start: offset }),
            (None, '“') => Some(Opening::Curly { start: offset }),
            (Some(Opening::Straight { start }), '"') | (Some(Opening::Curly { start }), '”') => {
                spans.push(span(start, offset, character));
                None
            }
            (None, '”') | (Some(_), '"' | '“' | '”') => {
                return ProtectedSpans::Indeterminate;
            }
            (current, _) => current,
        };
    }

    if opening.is_some() {
        ProtectedSpans::Indeterminate
    } else {
        ProtectedSpans::Exact(spans)
    }
}

impl ProtectedSpans {
    pub const fn is_indeterminate(&self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    pub fn protects(&self, start: usize, end: usize) -> bool {
        match self {
            Self::Exact(spans) => spans
                .iter()
                .any(|span| span.content_start <= start && end <= span.content_end),
            Self::Indeterminate => true,
        }
    }

    pub fn contains_offset(&self, offset: usize) -> bool {
        match self {
            Self::Exact(spans) => spans
                .iter()
                .any(|span| span.start <= offset && offset < span.end),
            Self::Indeterminate => false,
        }
    }

    pub fn quotation_at(&self, start: usize) -> Option<ProtectedSpan> {
        match self {
            Self::Exact(spans) => spans.iter().copied().find(|span| span.start == start),
            Self::Indeterminate => None,
        }
    }
}

const fn span(start: usize, content_end: usize, closing: char) -> ProtectedSpan {
    ProtectedSpan {
        start,
        content_start: start + if closing == '"' { 1 } else { '“'.len_utf8() },
        content_end,
        end: content_end + closing.len_utf8(),
    }
}
