pub fn obvious_string_is_open(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut double_quote_open = false;
    let mut backtick_open = false;
    let mut index = 0;
    while index < bytes.len() {
        if !double_quote_open && !backtick_open && bytes[index] == b'\'' {
            if let Some(end) = single_quote_end(bytes, index + 1) {
                index = end + 1;
                continue;
            }
        }
        match bytes[index] {
            b'\\' if !backtick_open => index = (index + 2).min(bytes.len()),
            b'"' if !backtick_open => {
                double_quote_open = !double_quote_open;
                index += 1;
            }
            b'`' if !double_quote_open => {
                backtick_open = !backtick_open;
                index += 1;
            }
            _ => index += 1,
        }
    }
    double_quote_open || backtick_open
}

fn single_quote_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'\'' => return Some(index),
            _ => index += 1,
        }
    }
    None
}
