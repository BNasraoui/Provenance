pub fn marker_is_inside_quoted_region(text: &str, marker: usize, track_backticks: bool) -> bool {
    let bytes = text.as_bytes();
    let mut quote = None;
    let mut index = 0;
    while index <= marker && index < bytes.len() {
        if index == marker {
            return quote.is_some();
        }
        if let Some(active_quote) = quote {
            match bytes[index] {
                b'\\' if active_quote != b'`' => index = (index + 2).min(bytes.len()),
                byte if byte == active_quote => {
                    quote = None;
                    index += 1;
                }
                _ => index += 1,
            }
            continue;
        }
        let candidate = bytes[index];
        if (matches!(candidate, b'\'' | b'"') || (track_backticks && candidate == b'`'))
            && quote_end(bytes, index + 1, candidate).is_some()
        {
            quote = Some(candidate);
        }
        index += 1;
    }
    false
}

fn quote_end(bytes: &[u8], mut index: usize, quote: u8) -> Option<usize> {
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if quote != b'`' => index += 2,
            byte if byte == quote => return Some(index),
            _ => index += 1,
        }
    }
    None
}
