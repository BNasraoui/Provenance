//! Deterministic descriptive-text checks from ASD-STE100 Issue 9.

mod contracted_verbs;
mod dictionary;
mod paragraph;
mod protected_spans;

pub use dictionary::{
    import_dictionary, DictionaryEntry, DictionaryImport, DictionaryImportError,
    DictionaryImportIdentity, DictionaryStatus, PartOfSpeech,
};

use provenance_macros::rule;
use serde::{Deserialize, Serialize};

pub(crate) mod sentence;
pub(crate) mod word_count;

/// The analyzer implementation version included in every report.
pub const ANALYZER_VERSION: &str = env!("CARGO_PKG_VERSION");

const SEMICOLON_MESSAGE: &str = "Do not use semicolons in descriptive text.";
const CONTRACTED_VERB_MESSAGE: &str = "Use the full verb form in descriptive text.";
const SENTENCE_LENGTH_MESSAGE: &str = "This descriptive sentence has more than 25 words.";
const PARAGRAPH_LENGTH_MESSAGE: &str = "This paragraph has more than six sentences.";

/// A report from the fixed ASD-STE100 Issue 9 analyzer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub standard: Standard,
    pub issue: StandardIssue,
    pub analyzer_version: String,
    pub findings: Vec<Finding>,
}

/// The authority used by the analyzer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Standard {
    #[serde(rename = "ASD-STE100")]
    AsdSte100,
}

/// The fixed issue of the standard used by the analyzer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum StandardIssue {
    Nine,
}

impl From<StandardIssue> for u8 {
    fn from(issue: StandardIssue) -> Self {
        match issue {
            StandardIssue::Nine => 9,
        }
    }
}

impl TryFrom<u8> for StandardIssue {
    type Error = &'static str;

    fn try_from(issue: u8) -> Result<Self, Self::Error> {
        match issue {
            9 => Ok(Self::Nine),
            _ => Err("the analyzer supports only ASD-STE100 Issue 9"),
        }
    }
}

/// One nonconformance found in descriptive text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub rule: RuleNumber,
    pub kind: FindingKind,
    pub span: Span,
    pub message: String,
}

/// An ASD-STE100 Issue 9 rule implemented by this analyzer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RuleNumber {
    #[serde(rename = "4.2")]
    FourTwo,
    #[serde(rename = "6.3")]
    SixThree,
    #[serde(rename = "6.6")]
    SixSix,
    #[serde(rename = "8.1")]
    EightOne,
}

/// The disposition of a finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Violation,
}

/// A half-open UTF-8 byte range in the analyzed text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Reports implemented, determinate ASD-STE100 violations in descriptive text.
#[rule("rule_ste100_semicolon")]
pub fn check_descriptive(text: &str) -> Report {
    let protected = protected_spans::analyze(text);
    let mut findings: Vec<_> = text
        .match_indices(';')
        .filter(|(start, _)| !protected.protects(*start, *start + 1))
        .map(|(start, _)| Finding {
            rule: RuleNumber::EightOne,
            kind: FindingKind::Violation,
            span: Span {
                start,
                end: start + 1,
            },
            message: SEMICOLON_MESSAGE.to_owned(),
        })
        .collect::<Vec<_>>();

    findings.extend(
        contracted_verbs::find(text, &protected)
            .into_iter()
            .map(|contracted| Finding {
                rule: RuleNumber::FourTwo,
                kind: FindingKind::Violation,
                span: Span {
                    start: contracted.start,
                    end: contracted.end,
                },
                message: CONTRACTED_VERB_MESSAGE.to_owned(),
            }),
    );
    findings.extend(sentence_length_findings(text, &protected));
    findings.extend(paragraph_sentence_findings(text, &protected));
    findings.sort_by_key(|finding| {
        (
            finding.span.start,
            finding.span.end,
            match finding.rule {
                RuleNumber::FourTwo => 0,
                RuleNumber::SixThree => 1,
                RuleNumber::SixSix => 2,
                RuleNumber::EightOne => 3,
            },
        )
    });

    Report {
        standard: Standard::AsdSte100,
        issue: StandardIssue::Nine,
        analyzer_version: ANALYZER_VERSION.to_owned(),
        findings,
    }
}

/// Reports each paragraph with clear boundaries that has more than six sentences.
#[rule("rule_ste100_paragraph_sentence_limit")]
fn paragraph_sentence_findings(
    text: &str,
    protected: &protected_spans::ProtectedSpans,
) -> Vec<Finding> {
    let Some(paragraphs) = paragraph::scan(text, protected) else {
        return Vec::new();
    };

    paragraphs
        .into_iter()
        .filter_map(|paragraph| {
            let sentences = sentence::scan_range(text, protected, paragraph.clone());
            (sentences.len() > 6 && sentences.iter().all(|sentence| !sentence.indeterminate)).then(
                || Finding {
                    rule: RuleNumber::SixSix,
                    kind: FindingKind::Violation,
                    span: Span {
                        start: paragraph.start,
                        end: paragraph.end,
                    },
                    message: PARAGRAPH_LENGTH_MESSAGE.to_owned(),
                },
            )
        })
        .collect()
}

/// Reports each determinate descriptive sentence that has more than 25 words.
#[rule("rule_ste100_descriptive_sentence_length")]
fn sentence_length_findings(
    text: &str,
    protected: &protected_spans::ProtectedSpans,
) -> Vec<Finding> {
    sentence::scan(text, protected)
        .into_iter()
        .filter_map(
            |sentence| match word_count::count(text, sentence, protected) {
                word_count::Count::Exact(count) if count > 25 => Some(Finding {
                    rule: RuleNumber::SixThree,
                    kind: FindingKind::Violation,
                    span: Span {
                        start: sentence.start,
                        end: sentence.end,
                    },
                    message: SENTENCE_LENGTH_MESSAGE.to_owned(),
                }),
                word_count::Count::Exact(_) | word_count::Count::Indeterminate => None,
            },
        )
        .collect()
}
