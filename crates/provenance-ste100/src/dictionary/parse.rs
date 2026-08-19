use super::{
    layout::{Line, PageTable},
    DictionaryEntry, DictionaryImportError, DictionaryStatus, PartOfSpeech,
};

#[derive(Clone, Copy)]
struct Location {
    page_index: usize,
    y: f32,
}

pub(super) fn parse_entries(
    tables: &[PageTable],
) -> Result<Vec<DictionaryEntry>, DictionaryImportError> {
    let columns: [Vec<&Line>; 4] = std::array::from_fn(|column| {
        tables
            .iter()
            .flat_map(|table| table.columns[column].iter())
            .collect()
    });
    let mut starts = entry_starts(&columns[0]);
    repair_wrapped_starts(&mut starts, &columns);
    let mut entries = Vec::with_capacity(starts.len());

    for (index, start) in starts.iter().enumerate() {
        let end = starts.get(index + 1).copied();
        let cells = [
            headword_cell_text(&columns[0], *start, end),
            cell_text(&columns[1], *start, end),
            cell_text(&columns[2], *start, end),
            cell_text(&columns[3], *start, end),
        ];
        entries.push(parse_entry(&cells, *start)?);
    }

    Ok(entries)
}

fn repair_wrapped_starts(starts: &mut [Location], columns: &[Vec<&Line>; 4]) {
    for start in starts.iter_mut() {
        let current = *start;
        let Some(line_index) = columns[0]
            .iter()
            .position(|line| same_location(current, location_of(line)))
        else {
            continue;
        };
        let Some(prior) = line_index
            .checked_sub(1)
            .and_then(|prior| columns[0].get(prior))
        else {
            continue;
        };
        if prior.page_index == current.page_index
            && (current.y - prior.y).abs() <= 16.0
            && prior.starts_at_column
            && split_part_of_speech(&prior.text).is_none()
            && has_line_near(&columns[1], location_of(prior))
            && has_line_near(&columns[2], location_of(prior))
        {
            *start = location_of(prior);
        }
    }
}

fn has_line_near(lines: &[&Line], location: Location) -> bool {
    lines
        .iter()
        .any(|line| line.page_index == location.page_index && (line.y - location.y).abs() <= 6.0)
}

fn entry_starts(lines: &[&Line]) -> Vec<Location> {
    let mut starts = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some((prefix, _)) = split_part_of_speech(&line.text) else {
            continue;
        };
        let prior = lines.get(index.saturating_sub(1)).copied();
        let continues_wrapped_headword = prior.is_some_and(|prior| {
            prior.page_index == line.page_index && prior.text.trim_end().ends_with('-')
        });
        let starts_with_qualifier = prefix.trim_start().starts_with('(');
        let start_line =
            if prefix.trim().is_empty() || starts_with_qualifier || continues_wrapped_headword {
                prior.unwrap_or(line)
            } else {
                line
            };
        if !start_line.starts_at_column {
            continue;
        }
        let location = location_of(start_line);
        if starts
            .last()
            .is_none_or(|prior| !same_location(*prior, location))
        {
            starts.push(location);
        }
    }
    starts
}

const fn location_of(line: &Line) -> Location {
    Location {
        page_index: line.page_index,
        y: line.y,
    }
}

/// Joins the word-column lines of one entry. Before the part-of-speech
/// marker, a bare one-token line continues the next line without a space,
/// because the extractor can drop the printed hyphen at a line break.
/// Lines from the marker on list word forms and join with a space.
fn headword_cell_text(lines: &[&Line], start: Location, end: Option<Location>) -> String {
    let mut text = String::new();
    let mut marker_seen = false;
    let mut previous: Option<String> = None;
    let included = lines
        .iter()
        .filter(|line| at_or_after(line, start) && end.is_none_or(|end| before(line, end)));
    for line in included {
        let normalized = line.text.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.is_empty() {
            continue;
        }
        if let Some(previous) = &previous {
            let wraps_one_word = !marker_seen
                && !previous.contains('(')
                && !normalized.starts_with('(')
                && (previous.ends_with('-') || !previous.contains(' '));
            if wraps_one_word {
                if previous.ends_with('-') {
                    text.pop();
                }
            } else {
                text.push(' ');
            }
        }
        text.push_str(&normalized);
        marker_seen = marker_seen || split_part_of_speech(&normalized).is_some();
        previous = Some(normalized);
    }
    text
}

fn cell_text(lines: &[&Line], start: Location, end: Option<Location>) -> String {
    lines
        .iter()
        .filter(|line| at_or_after(line, start) && end.is_none_or(|end| before(line, end)))
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_entry(
    cells: &[String; 4],
    location: Location,
) -> Result<DictionaryEntry, DictionaryImportError> {
    let (headword_text, part_and_forms) = split_part_of_speech(&cells[0])
        .ok_or_else(|| malformed(location, "an entry has no recognized part of speech"))?;
    let (part, suffix) = parse_part_of_speech(part_and_forms)
        .ok_or_else(|| malformed(location, "an entry has an unknown part of speech"))?;
    let (headword, label_forms) = parse_headword_label(headword_text);
    let status = classify_status(&headword).ok_or_else(|| {
        malformed(
            location,
            "an entry headword is neither uppercase nor lowercase",
        )
    })?;
    let word_forms = std::iter::once(headword.clone())
        .chain(label_forms)
        .chain(
            suffix
                .trim_start_matches(',')
                .split(',')
                .map(str::trim)
                .filter(|form| !form.is_empty())
                .map(str::to_owned),
        )
        .collect::<Vec<_>>();
    let non_ste_example = (!cells[3].is_empty()).then(|| cells[3].clone());

    let example_free = part == PartOfSpeech::Prefix;
    let missing_column = if headword.is_empty() {
        Some("headword")
    } else if cells[1].is_empty() {
        Some("approved meaning or alternatives")
    } else if cells[2].is_empty() && !example_free {
        Some("STE example")
    } else if status == DictionaryStatus::Unapproved && non_ste_example.is_none() && !example_free {
        Some("non-STE example")
    } else {
        None
    };
    if let Some(column) = missing_column {
        return Err(malformed(
            location,
            &format!("an entry has no {column} column data"),
        ));
    }

    Ok(DictionaryEntry {
        headword,
        word_forms,
        part_of_speech: part,
        status,
        approved_meaning_or_alternatives: cells[1].clone(),
        ste_example: cells[2].clone(),
        non_ste_example,
    })
}

fn parse_headword_label(text: &str) -> (String, Vec<String>) {
    let text = text.trim().trim_end_matches(',');
    let Some(alternative_start) = text.find(" (or ") else {
        return parse_qualified_headword(text);
    };
    let Some(alternative_end) = text.rfind(')') else {
        return (text.to_owned(), Vec::new());
    };
    if alternative_end <= alternative_start + 5 {
        return (text.to_owned(), Vec::new());
    }

    let headword = text[..alternative_start].trim().to_owned();
    let forms = text[alternative_start + 5..alternative_end]
        .split(" or ")
        .flat_map(|form| form.split(','))
        .map(str::trim)
        .filter(|form| !form.is_empty())
        .map(str::to_owned)
        .collect();
    (headword, forms)
}

fn parse_qualified_headword(text: &str) -> (String, Vec<String>) {
    let Some(qualifier_start) = text.find(" (") else {
        return (text.to_owned(), Vec::new());
    };
    let Some(qualifier_end) = text.rfind(')') else {
        return (text.to_owned(), Vec::new());
    };
    if qualifier_end <= qualifier_start + 2 {
        return (text.to_owned(), Vec::new());
    }

    (
        text[..qualifier_start].trim().to_owned(),
        vec![text[qualifier_start + 2..qualifier_end].trim().to_owned()],
    )
}

fn split_part_of_speech(text: &str) -> Option<(&str, &str)> {
    const MARKERS: [&str; 9] = [
        "(adj)", "(adv)", "(art)", "(conj)", "(n)", "(prefix)", "(prep)", "(pron)", "(v)",
    ];
    MARKERS
        .iter()
        .filter_map(|marker| text.find(marker))
        .min()
        .map(|index| (&text[..index], &text[index..]))
}

fn parse_part_of_speech(text: &str) -> Option<(PartOfSpeech, &str)> {
    let close = text.find(')')?;
    let part = match &text[..=close] {
        "(adj)" => PartOfSpeech::Adjective,
        "(adv)" => PartOfSpeech::Adverb,
        "(art)" => PartOfSpeech::Article,
        "(conj)" => PartOfSpeech::Conjunction,
        "(n)" => PartOfSpeech::Noun,
        "(prefix)" => PartOfSpeech::Prefix,
        "(prep)" => PartOfSpeech::Preposition,
        "(pron)" => PartOfSpeech::Pronoun,
        "(v)" => PartOfSpeech::Verb,
        _ => return None,
    };
    Some((part, &text[close + 1..]))
}

fn classify_status(headword: &str) -> Option<DictionaryStatus> {
    let letters = headword
        .chars()
        .filter(|character| character.is_alphabetic());
    let letters = letters.collect::<Vec<_>>();
    if letters.is_empty() {
        None
    } else if letters.iter().all(|character| character.is_uppercase()) {
        Some(DictionaryStatus::Approved)
    } else if letters.iter().all(|character| character.is_lowercase()) {
        Some(DictionaryStatus::Unapproved)
    } else {
        None
    }
}

fn malformed(location: Location, reason: &str) -> DictionaryImportError {
    DictionaryImportError::InvalidStructure {
        approved_rows: 0,
        unapproved_rows: 0,
        reason: format!(
            "PDF page {} at row {:.1}: {reason}",
            location.page_index + 1,
            location.y
        ),
    }
}

fn at_or_after(line: &Line, location: Location) -> bool {
    line.page_index > location.page_index
        || (line.page_index == location.page_index && line.y + 2.5 >= location.y)
}

fn before(line: &Line, location: Location) -> bool {
    line.page_index < location.page_index
        || (line.page_index == location.page_index && line.y < location.y - 2.5)
}

fn same_location(left: Location, right: Location) -> bool {
    left.page_index == right.page_index && (left.y - right.y).abs() <= 2.5
}
