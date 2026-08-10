use std::{fmt, str::FromStr};

pub(crate) const PRIMARY_ANNOTATION_MARKER: &str = "@provenance";
const LEGACY_ANNOTATION_MARKER: &str = "@statesman";
const ANNOTATION_MARKERS: [&str; 2] = [PRIMARY_ANNOTATION_MARKER, LEGACY_ANNOTATION_MARKER];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLevel {
    #[default]
    Full,
    Partial,
    Indirect,
}

impl fmt::Display for CoverageLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Partial => write!(f, "partial"),
            Self::Indirect => write!(f, "indirect"),
        }
    }
}

impl FromStr for CoverageLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "full" => Ok(Self::Full),
            "partial" => Ok(Self::Partial),
            "indirect" => Ok(Self::Indirect),
            other => Err(format!("invalid coverage level: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verification {
    Exhaustion,
    Property,
    Examples,
    Conformance,
    Construction,
}

impl fmt::Display for Verification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhaustion => write!(f, "exhaustion"),
            Self::Property => write!(f, "property"),
            Self::Examples => write!(f, "examples"),
            Self::Conformance => write!(f, "conformance"),
            Self::Construction => write!(f, "construction"),
        }
    }
}

impl FromStr for Verification {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "exhaustion" => Ok(Self::Exhaustion),
            "property" => Ok(Self::Property),
            "examples" => Ok(Self::Examples),
            "conformance" => Ok(Self::Conformance),
            "construction" => Ok(Self::Construction),
            other => Err(format!("invalid verification method: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Annotation {
    pub rule: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub coverage: CoverageLevel,
    pub confidence: f64,
    pub intent: Option<String>,
    pub verification: Option<Verification>,
}

impl Default for Annotation {
    fn default() -> Self {
        Self {
            rule: String::new(),
            name: None,
            description: None,
            tags: Vec::new(),
            coverage: CoverageLevel::Full,
            confidence: 1.0,
            intent: None,
            verification: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ParseWarning {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    pub annotations: Vec<Annotation>,
    pub warnings: Vec<ParseWarning>,
}

pub fn parse_annotations(comment_text: &str) -> ParseResult {
    let mut warnings = Vec::new();
    let mut annotations: Vec<Annotation> = Vec::new();
    let mut shared = Annotation::default();
    let mut saw_directive_marker = None;

    for (line_idx, raw_line) in comment_text.lines().enumerate() {
        let line = line_idx + 1;
        let stripped = strip_comment_prefix(raw_line);
        let Some((marker, after_marker)) = split_annotation_marker(stripped) else {
            continue;
        };
        saw_directive_marker.get_or_insert(marker);
        let Some((key, value)) = after_marker.split_once(':') else {
            warnings.push(ParseWarning {
                line,
                message: format!("malformed directive: expected `key: value` after {marker}"),
            });
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if value.is_empty() && key != "tags" {
            warnings.push(ParseWarning {
                line,
                message: format!("empty value for field `{}`", key.trim()),
            });
            continue;
        }
        match key.as_str() {
            "rule" => annotations.push(Annotation {
                rule: value.to_string(),
                name: shared.name.clone(),
                description: shared.description.clone(),
                tags: shared.tags.clone(),
                coverage: shared.coverage,
                confidence: shared.confidence,
                intent: shared.intent.clone(),
                verification: shared.verification,
            }),
            "name" => set_field(&mut annotations, &mut shared, |ann| {
                ann.name = Some(value.to_string());
            }),
            "description" => set_field(&mut annotations, &mut shared, |ann| {
                ann.description = Some(value.to_string());
            }),
            "tags" => {
                let tags = value
                    .split(',')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                set_field(&mut annotations, &mut shared, |ann| {
                    ann.tags.clone_from(&tags);
                });
            }
            "coverage" => match CoverageLevel::from_str(value) {
                Ok(level) => set_field(&mut annotations, &mut shared, |ann| ann.coverage = level),
                Err(_) => warnings.push(ParseWarning {
                    line,
                    message: format!("invalid coverage level `{value}`, using default"),
                }),
            },
            "confidence" => match parse_confidence(value) {
                Ok(confidence) => set_field(&mut annotations, &mut shared, |ann| {
                    ann.confidence = confidence;
                }),
                Err(rejection) => warnings.push(ParseWarning {
                    line,
                    message: rejection.warning(value),
                }),
            },
            "intent" => set_field(&mut annotations, &mut shared, |ann| {
                ann.intent = Some(value.to_string());
            }),
            "verification" => match Verification::from_str(value) {
                Ok(method) => set_field(&mut annotations, &mut shared, |ann| {
                    ann.verification = Some(method);
                }),
                Err(_) => warnings.push(ParseWarning {
                    line,
                    message: format!("invalid verification method `{value}`, ignoring"),
                }),
            },
            other => warnings.push(ParseWarning {
                line,
                message: format!("unknown field `{other}`"),
            }),
        }
    }

    if let Some(marker) = saw_directive_marker.filter(|_| annotations.is_empty()) {
        warnings.push(ParseWarning {
            line: 0,
            message: format!("found {marker} directives but no rule annotations"),
        });
    }

    ParseResult {
        annotations,
        warnings,
    }
}

/// Why an annotated confidence was not used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfidenceRejection {
    /// The text is not a number, or is a NaN or an infinity.
    Unreadable,
    /// A real number, but not one between 0.0 and 1.0.
    OutOfRange,
}

impl ConfidenceRejection {
    fn warning(self, value: &str) -> String {
        match self {
            Self::Unreadable => format!("invalid confidence `{value}`, using default"),
            Self::OutOfRange => format!("confidence `{value}` is outside 0.0-1.0, using default"),
        }
    }
}

/// The scanner's copy of `rule_confidence_range`, whose deciding function is
/// `validate_confidence_score` in `provenance-core/src/model/validation.rs`.
/// The scanner needs the range before any record exists, so it holds the range
/// itself; it must agree with core on the verdict and may differ only in what
/// it does about it. The conformance test in
/// `provenance-scanner/tests/confidence_conformance.rs` holds the two in step.
fn confidence_in_range(confidence: f64) -> bool {
    confidence.is_finite() && (0.0..=1.0).contains(&confidence)
}

/// Reads an annotated confidence, refusing anything outside 0.0 to 1.0.
///
/// An out-of-range score is refused rather than pulled to the nearest end: the
/// caller warns and keeps the default, so a reader of the report sees that the
/// annotation said something the graph would not accept, instead of a 1.0 that
/// looks like a choice somebody made.
fn parse_confidence(value: &str) -> Result<f64, ConfidenceRejection> {
    let confidence = value
        .parse::<f64>()
        .ok()
        .filter(|confidence| confidence.is_finite())
        .ok_or(ConfidenceRejection::Unreadable)?;
    if confidence_in_range(confidence) {
        Ok(confidence)
    } else {
        Err(ConfidenceRejection::OutOfRange)
    }
}

pub(crate) fn contains_annotation_marker(line: &str) -> bool {
    ANNOTATION_MARKERS
        .iter()
        .any(|marker| line.contains(marker))
}

fn split_annotation_marker(line: &str) -> Option<(&'static str, &str)> {
    ANNOTATION_MARKERS.iter().find_map(|marker| {
        line.split_once(marker)
            .map(|(_, rest)| (*marker, rest.trim()))
    })
}

fn set_field<F>(annotations: &mut [Annotation], shared: &mut Annotation, apply: F)
where
    F: Fn(&mut Annotation),
{
    if let Some(annotation) = annotations.last_mut() {
        apply(annotation);
    } else {
        apply(shared);
    }
}

fn strip_comment_prefix(line: &str) -> &str {
    line.trim_start()
        .trim_start_matches('/')
        .trim_start_matches('*')
        .trim_start_matches('#')
        .trim_start_matches('-')
        .trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shared_fields_across_multiple_provenance_rules() {
        let parsed = parse_annotations(
            r"
            @provenance name: Payroll thresholds
            @provenance coverage: full
            @provenance rule: SCHADS-PAY-001
            @provenance rule: SCHADS-PAY-002
            ",
        );

        assert_eq!(parsed.annotations.len(), 2);
        assert_eq!(
            parsed.annotations[0].name.as_deref(),
            Some("Payroll thresholds")
        );
        assert_eq!(parsed.annotations[1].coverage, CoverageLevel::Full);
    }

    #[test]
    fn parses_statesman_marker_as_legacy_alias() {
        let parsed = parse_annotations("@statesman rule: SCHADS-PAY-001");

        assert_eq!(parsed.annotations.len(), 1);
        assert_eq!(parsed.annotations[0].rule, "SCHADS-PAY-001");
    }

    #[test]
    fn rejects_non_finite_confidence_with_warning() {
        for value in ["NaN", "inf", "-inf"] {
            let parsed = parse_annotations(&format!(
                "@provenance confidence: {value}\n@provenance rule: RULE-001"
            ));

            assert!((parsed.annotations[0].confidence - 1.0).abs() < f64::EPSILON);
            assert_eq!(
                parsed.warnings,
                vec![ParseWarning {
                    line: 1,
                    message: format!("invalid confidence `{value}`, using default"),
                }]
            );
        }
    }

    /// Out of range is refused, not pulled to the nearest end: the graph
    /// rejects such a score outright (`rule_confidence_range`), and a scan
    /// that quietly clamped would report a confidence nobody wrote.
    #[test]
    fn refuses_out_of_range_confidence_and_keeps_the_default() {
        for value in ["-0.25", "1.25", "-1", "2", "100"] {
            let parsed = parse_annotations(&format!(
                "@provenance confidence: {value}\n@provenance rule: RULE-001"
            ));

            assert!((parsed.annotations[0].confidence - 1.0).abs() < f64::EPSILON);
            assert_eq!(
                parsed.warnings,
                vec![ParseWarning {
                    line: 1,
                    message: format!("confidence `{value}` is outside 0.0-1.0, using default"),
                }]
            );
        }
    }

    #[test]
    fn preserves_in_range_confidence() {
        let parsed = parse_annotations("@provenance confidence: 0.75\n@provenance rule: RULE-001");

        assert!((parsed.annotations[0].confidence - 0.75).abs() < f64::EPSILON);
        assert!(parsed.warnings.is_empty());
    }
}
