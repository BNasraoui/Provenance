use provenance_macros::rule;

#[derive(Clone, Copy, Debug)]
pub struct Sentence {
    pub start: usize,
    pub end: usize,
    pub indeterminate: bool,
}

/// Keeps ordinary colons within a sentence and leaves list-like input indeterminate.
#[rule("rule_ste100_ordinary_colon_continuity")]
pub fn scan(text: &str) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut sentence_start = 0;
    let mut parentheses = Vec::new();
    let mut quote_end = None;
    let mut top_level_indeterminate = false;

    for (offset, character) in text.char_indices() {
        match character {
            '"' if quote_end.is_none() => quote_end = Some('"'),
            '"' if quote_end == Some('"') => quote_end = None,
            '“' if quote_end.is_none() => quote_end = Some('”'),
            '”' if quote_end == Some('”') => quote_end = None,
            '”' if quote_end.is_none() => top_level_indeterminate = true,
            '(' if quote_end.is_none() => {
                if !parentheses.is_empty() {
                    top_level_indeterminate = true;
                }
                parentheses.push(offset);
            }
            ')' if quote_end.is_none() => {
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
                if quote_end.is_none()
                    && parentheses.is_empty()
                    && (character != '.' || !is_decimal_point(text, offset)) =>
            {
                let end = offset + character.len_utf8();
                push_trimmed(
                    text,
                    sentence_start,
                    end,
                    top_level_indeterminate,
                    &mut sentences,
                );
                sentence_start = end;
                top_level_indeterminate = false;
            }
            _ => {}
        }
    }

    top_level_indeterminate |= quote_end.is_some() || !parentheses.is_empty();
    push_trimmed(
        text,
        sentence_start,
        text.len(),
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
