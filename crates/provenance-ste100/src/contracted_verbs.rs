use provenance_macros::rule;

use crate::protected_spans::ProtectedSpans;

pub struct ContractedVerb {
    pub start: usize,
    pub end: usize,
}

/// Finds contracted verb tokens whose classification needs no grammar or meaning choice.
#[rule("rule_ste100_contracted_verb")]
pub fn find(text: &str, protected: &ProtectedSpans) -> Vec<ContractedVerb> {
    token_ranges(text)
        .filter(|&(start, end)| !protected.protects(start, end) && is_recognized(&text[start..end]))
        .map(|(start, end)| ContractedVerb { start, end })
        .collect()
}

fn token_ranges(text: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let mut characters = text.char_indices().peekable();

    std::iter::from_fn(move || {
        while let Some((_, character)) = characters.peek() {
            if is_token_character(*character) {
                break;
            }
            characters.next();
        }

        let (start, _) = characters.next()?;
        let mut end = text.len();
        while let Some(&(offset, character)) = characters.peek() {
            if !is_token_character(character) {
                end = offset;
                break;
            }
            characters.next();
        }
        Some((start, end))
    })
}

fn is_token_character(character: char) -> bool {
    character.is_alphanumeric()
        || matches!(character, '_' | '\'' | '\u{2018}' | '\u{2019}' | '\u{2032}')
}

fn is_recognized(token: &str) -> bool {
    let normalized = token
        .chars()
        .map(|character| match character {
            '\u{2019}' => '\'',
            other => other.to_ascii_lowercase(),
        })
        .collect::<String>();

    matches!(
        normalized.as_str(),
        "i'm"
            | "you're"
            | "we're"
            | "they're"
            | "who're"
            | "what're"
            | "there're"
            | "i've"
            | "you've"
            | "we've"
            | "they've"
            | "who've"
            | "what've"
            | "could've"
            | "should've"
            | "would've"
            | "might've"
            | "must've"
            | "i'll"
            | "you'll"
            | "he'll"
            | "she'll"
            | "it'll"
            | "we'll"
            | "they'll"
            | "that'll"
            | "who'll"
            | "what'll"
            | "there'll"
            | "i'd"
            | "you'd"
            | "he'd"
            | "she'd"
            | "it'd"
            | "we'd"
            | "they'd"
            | "that'd"
            | "who'd"
            | "what'd"
            | "where'd"
            | "when'd"
            | "why'd"
            | "how'd"
            | "there'd"
            | "he's"
            | "she's"
            | "it's"
            | "that's"
            | "what's"
            | "who's"
            | "where's"
            | "when's"
            | "why's"
            | "how's"
            | "there's"
            | "here's"
            | "ain't"
            | "amn't"
            | "aren't"
            | "can't"
            | "couldn't"
            | "daren't"
            | "didn't"
            | "doesn't"
            | "don't"
            | "hadn't"
            | "hasn't"
            | "haven't"
            | "isn't"
            | "mayn't"
            | "mightn't"
            | "mustn't"
            | "needn't"
            | "oughtn't"
            | "shan't"
            | "shouldn't"
            | "wasn't"
            | "weren't"
            | "won't"
            | "wouldn't"
    )
}
