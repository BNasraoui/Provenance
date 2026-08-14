//! Deterministic descriptive-text checks from ASD-STE100 Issue 9.

mod contracted_verbs;

use provenance_macros::rule;
use serde::{Deserialize, Serialize};

/// The analyzer implementation version included in every report.
pub const ANALYZER_VERSION: &str = env!("CARGO_PKG_VERSION");

const SEMICOLON_MESSAGE: &str = "Do not use semicolons in descriptive text.";
const CONTRACTED_VERB_MESSAGE: &str = "Use the full verb form in descriptive text.";

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

/// Reports deterministic Rule 4.2 and Rule 8.1 violations in source order.
#[rule("rule_ste100_semicolon")]
pub fn check_descriptive(text: &str) -> Report {
    let mut findings = text
        .match_indices(';')
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
        contracted_verbs::find(text)
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
    findings.sort_by_key(|finding| finding.span.start);

    Report {
        standard: Standard::AsdSte100,
        issue: StandardIssue::Nine,
        analyzer_version: ANALYZER_VERSION.to_owned(),
        findings,
    }
}
