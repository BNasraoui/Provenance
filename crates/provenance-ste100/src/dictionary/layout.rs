use super::{pdf::DictionaryPage, DictionaryImportError};

const LINE_TOLERANCE: f32 = 2.5;

#[derive(Clone, Debug)]
pub(super) struct Line {
    pub page_index: usize,
    pub y: f32,
    pub text: String,
    pub starts_at_column: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PageTable {
    pub columns: [Vec<Line>; 4],
}

pub(super) fn table_from_page(page: &DictionaryPage) -> Result<PageTable, DictionaryImportError> {
    let [word_x, meaning_x, ste_x, non_ste_x, header_y] =
        find_header(page).ok_or_else(|| DictionaryImportError::InvalidStructure {
            approved_rows: 0,
            unapproved_rows: 0,
            reason: format!("PDF page {} has no four-column header", page.page_index + 1),
        })?;
    let boundaries = [meaning_x - 30.0, ste_x - 20.0, non_ste_x - 20.0];
    let footer_y = page
        .marker_words
        .iter()
        .filter(|word| matches!(normalized(&word.text).as_str(), "PAGE" | "ISSUE"))
        .max_by(|left, right| {
            (left.y - header_y)
                .abs()
                .total_cmp(&(right.y - header_y).abs())
        })
        .map(|word| word.y)
        .ok_or_else(|| DictionaryImportError::InvalidStructure {
            approved_rows: 0,
            unapproved_rows: 0,
            reason: format!("PDF page {} has no dictionary footer", page.page_index + 1),
        })?;
    let reads_toward_smaller_y = footer_y < header_y;
    let reading_y = |y: f32| if reads_toward_smaller_y { -y } else { y };
    let header_y = reading_y(header_y);
    let footer_y = reading_y(footer_y);
    let mut column_words: [Vec<_>; 4] = std::array::from_fn(|_| Vec::new());

    for word in &page.words {
        let y = reading_y(word.y);
        if y <= header_y + 16.0 || y >= footer_y - 3.0 {
            continue;
        }
        let column = if word.x < boundaries[0] {
            0
        } else if word.x < meaning_x {
            usize::from(!word.is_bold)
        } else {
            1 + boundaries[1..]
                .iter()
                .take_while(|edge| word.x >= **edge)
                .count()
        };
        column_words[column].push((word, y));
    }

    Ok(PageTable {
        columns: std::array::from_fn(|column| {
            make_lines(
                page.page_index,
                std::mem::take(&mut column_words[column]),
                (column == 0).then_some(word_x),
            )
        }),
    })
}

fn find_header(page: &DictionaryPage) -> Option<[f32; 5]> {
    let word_candidates = page
        .words
        .iter()
        .filter(|word| normalized(&word.text).starts_with("WORD"));

    for word in word_candidates {
        let Some(approved) = nearby_after(page, word, "APPROVED", 5.0) else {
            continue;
        };
        let Some(ste) = nearby_after(page, approved, "STE", 15.0) else {
            continue;
        };
        let Some(non_ste) = nearby_after(page, ste, "NON-STE", 5.0) else {
            continue;
        };
        return Some([word.x, approved.x, ste.x, non_ste.x, word.y]);
    }
    None
}

fn nearby_after<'a>(
    page: &'a DictionaryPage,
    prior: &super::pdf::PositionedWord,
    prefix: &str,
    y_tolerance: f32,
) -> Option<&'a super::pdf::PositionedWord> {
    page.words.iter().find(|candidate| {
        candidate.x > prior.x
            && (candidate.y - prior.y).abs() <= y_tolerance
            && normalized(&candidate.text).starts_with(prefix)
    })
}

fn make_lines(
    page_index: usize,
    mut words: Vec<(&super::pdf::PositionedWord, f32)>,
    column_start: Option<f32>,
) -> Vec<Line> {
    words.sort_by(|left, right| {
        left.1
            .total_cmp(&right.1)
            .then_with(|| left.0.x.total_cmp(&right.0.x))
    });
    let mut groups: Vec<Vec<(&super::pdf::PositionedWord, f32)>> = Vec::new();
    for word in words {
        match groups.last_mut() {
            Some(group) if (word.1 - group[0].1).abs() <= LINE_TOLERANCE => group.push(word),
            _ => groups.push(vec![word]),
        }
    }

    groups
        .into_iter()
        .map(|mut group| {
            group.sort_by(|left, right| left.0.x.total_cmp(&right.0.x));
            Line {
                page_index,
                y: group[0].1,
                starts_at_column: column_start
                    .is_some_and(|start| (group[0].0.x - start).abs() <= 5.0),
                text: {
                    let tokens = merge_split_tokens(&group);
                    join_words(tokens.iter().map(String::as_str))
                },
            }
        })
        .collect()
}

/// Rejoins tokens the extractor split inside one printed word. A real word
/// gap in the dictionary is two points or wider; a split from a kerning
/// offset leaves no gap at all.
fn merge_split_tokens(group: &[(&super::pdf::PositionedWord, f32)]) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut previous_end = f32::NEG_INFINITY;
    for (word, _) in group {
        match tokens.last_mut() {
            Some(last) if word.x - previous_end < 1.0 => last.push_str(&word.text),
            _ => tokens.push(word.text.clone()),
        }
        previous_end = previous_end.max(word.x_end);
    }
    tokens
}

fn join_words<'a>(words: impl Iterator<Item = &'a str>) -> String {
    let joined = words.collect::<Vec<_>>().join(" ");
    joined
        .replace(" .", ".")
        .replace(" ,", ",")
        .replace(" ;", ";")
        .replace(" :", ":")
        .replace(" )", ")")
        .replace("( ", "(")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized(text: &str) -> String {
    text.trim().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::super::pdf::PositionedWord;
    use super::make_lines;

    fn word(text: &str, x: f32, x_end: f32) -> PositionedWord {
        PositionedWord {
            text: text.to_owned(),
            x,
            x_end,
            y: 100.0,
            is_bold: true,
        }
    }

    #[test]
    fn joins_a_word_split_between_two_tokens_without_a_space() {
        let words = [
            word("by", 49.8, 62.7),
            word("means", 65.8, 100.7),
            word("of", 103.7, 114.1),
            word("(pre", 117.1, 138.0),
            word("p)", 138.0, 148.3),
        ];
        let row = words.iter().map(|word| (word, 100.0)).collect();

        let lines = make_lines(0, row, Some(49.8));

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "by means of (prep)");
    }
}
