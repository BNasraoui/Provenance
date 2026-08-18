use pdf_oxide::{fonts::MappingProvenance, PdfDocument};
use provenance_macros::rule;

use super::DictionaryImportError;

#[derive(Clone, Debug)]
pub(super) struct PositionedWord {
    pub text: String,
    pub x: f32,
    pub x_end: f32,
    pub y: f32,
    pub is_bold: bool,
}

#[derive(Clone, Debug)]
pub(super) struct DictionaryPage {
    pub page_index: usize,
    pub words: Vec<PositionedWord>,
    pub marker_words: Vec<PositionedWord>,
}

/// Accepts only a decodable Issue 9 PDF with Part 2 dictionary pages.
#[rule("rule_ste_dictionary_pdf_validation")]
pub(super) fn extract_dictionary_pages(
    bytes: &[u8],
) -> Result<Vec<DictionaryPage>, DictionaryImportError> {
    let document = PdfDocument::from_bytes(bytes.to_vec()).map_err(|error| {
        DictionaryImportError::InvalidPdf {
            message: error.to_string(),
        }
    })?;
    let mut saw_identity = false;
    let mut saw_word_totals = false;
    let mut pages = Vec::new();

    for page_index in document.page_indices() {
        let raw_words = document
            .extract_words_with_thresholds(page_index, None, None)
            .map_err(|error| DictionaryImportError::UnreliableText {
                page: page_index + 1,
                reason: error.to_string(),
            })?;
        let marker_words = raw_words.iter().map(positioned).collect::<Vec<_>>();
        let compact_text = compact_page_text(&marker_words);

        if identifies_issue_9(&compact_text) {
            saw_identity = true;
        }
        if compact_text.contains("875APPROVEDWORDS") && compact_text.contains("1274WORDS") {
            saw_word_totals = true;
        }
        if compact_text.contains("PAGE2-1-") {
            let spans = document.extract_spans(page_index).map_err(|error| {
                DictionaryImportError::UnreliableText {
                    page: page_index + 1,
                    reason: error.to_string(),
                }
            })?;
            if spans
                .iter()
                .any(|span| matches!(span.provenance, Some(MappingProvenance::Fallback)))
            {
                return Err(DictionaryImportError::UnreliableText {
                    page: page_index + 1,
                    reason: "the PDF decoder used fallback character mappings".to_owned(),
                });
            }
            pages.push(DictionaryPage {
                page_index,
                words: marker_words.clone(),
                marker_words,
            });
        }
    }

    if !saw_identity {
        return Err(DictionaryImportError::UnsupportedDocument {
            reason: "the PDF does not identify ASD-STE100 Issue 9".to_owned(),
        });
    }
    if !saw_word_totals {
        return Err(DictionaryImportError::UnsupportedDocument {
            reason: "the PDF does not state the Issue 9 dictionary word totals".to_owned(),
        });
    }
    if pages.is_empty() {
        return Err(DictionaryImportError::DictionaryNotFound);
    }

    Ok(pages)
}

fn positioned(word: &pdf_oxide::layout::Word) -> PositionedWord {
    PositionedWord {
        text: word.text.clone(),
        x: word.bbox.x,
        x_end: word.bbox.x + word.bbox.width,
        y: word.bbox.y,
        is_bold: word.is_bold,
    }
}

fn compact_page_text(words: &[PositionedWord]) -> String {
    words
        .iter()
        .flat_map(|word| word.text.chars())
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect()
}

fn identifies_issue_9(text: &str) -> bool {
    let standard = text.contains("ASD-STE100SIMPLIFIEDTECHNICALENGLISH")
        || text.contains("ASDSTE100SIMPLIFIEDTECHNICALENGLISH");
    standard && text.contains("ISSUE9")
}
