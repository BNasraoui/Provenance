pub(super) fn tokens(text: &str) -> Vec<(usize, &str)> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (index, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if let Some(begin) = start.take() {
                tokens.push((begin, &text[begin..index]));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(begin) = start {
        tokens.push((begin, &text[begin..]));
    }
    tokens
}

pub(super) fn trim_token(token: &str) -> (usize, &str) {
    const LEADING: &[char] = &['(', '[', '{', '"', '\'', '<'];
    const TRAILING: &[char] = &['.', ',', ';', ':', '!', '?', ')', ']', '}', '"', '\'', '>'];
    let trimmed = token.trim_start_matches(LEADING);
    let offset = token.len() - trimmed.len();
    (offset, trimmed.trim_end_matches(TRAILING))
}

pub(super) fn is_test_name(token: &str) -> bool {
    token.strip_prefix("test").is_some_and(|rest| {
        rest.starts_with(|ch: char| ch.is_ascii_uppercase())
            && rest
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    })
}
