use serde::{de::Error, Deserialize, Deserializer};
use std::collections::BTreeSet;

use super::{IdeationEvidenceReference, ScopeId, StableId};

pub fn validate_confidence_score(confidence: f64) -> anyhow::Result<()> {
    anyhow::ensure!(
        confidence.is_finite() && (0.0..=1.0).contains(&confidence),
        "confidence must be between 0.0 and 1.0"
    );
    Ok(())
}

pub fn validate_optional_confidence_score(confidence: Option<f64>) -> anyhow::Result<Option<f64>> {
    match confidence {
        Some(confidence) => {
            validate_confidence_score(confidence)?;
            Ok(Some(confidence))
        }
        None => Ok(None),
    }
}

pub(super) fn deserialize_optional_confidence<'de, D>(
    deserializer: D,
) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let confidence = Option::<f64>::deserialize(deserializer)?;
    validate_optional_confidence_score(confidence).map_err(D::Error::custom)
}

pub fn validate_commit_pin(commit_pin: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        (7..=64).contains(&commit_pin.len())
            && commit_pin.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "commit pin must be a 7-64 character hexadecimal Git commit"
    );
    Ok(())
}

pub fn validate_optional_commit_pin(commit_pin: Option<String>) -> anyhow::Result<Option<String>> {
    match commit_pin {
        Some(commit_pin) => {
            validate_commit_pin(&commit_pin)?;
            Ok(Some(commit_pin))
        }
        None => Ok(None),
    }
}

pub(super) fn deserialize_optional_commit_pin<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let commit_pin = Option::<String>::deserialize(deserializer)?;
    validate_optional_commit_pin(commit_pin).map_err(D::Error::custom)
}

pub fn validate_evidence_references(
    references: &[IdeationEvidenceReference],
) -> anyhow::Result<()> {
    for reference in references {
        anyhow::ensure!(
            reference.line.is_none_or(|line| line >= 1),
            "evidence line must be at least 1 for {}",
            reference.reference_id.as_str()
        );
    }
    Ok(())
}

pub fn validate_record_scope(
    expected: &ScopeId,
    actual: &ScopeId,
    kind: &str,
    id: &StableId,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        actual == expected,
        "{kind} {} belongs to scope {}, expected {}",
        id.as_str(),
        actual.as_str(),
        expected.as_str()
    );
    Ok(())
}

pub fn validate_unique_ids<'a>(
    kind: &str,
    ids: impl IntoIterator<Item = &'a StableId>,
) -> anyhow::Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        anyhow::ensure!(
            seen.insert(id.as_str()),
            "duplicate {kind} id {}",
            id.as_str()
        );
    }
    Ok(())
}
