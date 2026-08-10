use crate::skills;
use anyhow::Context;
use camino::Utf8PathBuf;
use provenance_core::{
    validate_resolution_input_content, CanonicalArtifact, CanonicalArtifactType, IdeationTarget,
    IdeationTargetType, ResolutionInput, ResolutionInputType, SourceReference, StableId,
};
use provenance_macros::rule;
use serde::de::DeserializeOwned;
use std::borrow::Cow;

/// A family of flags names one kind of record, one field per flag, and
/// repeating the whole family names several records. The family is complete
/// when every flag naming a required field was given once per record, and
/// every flag naming an optional field was either omitted altogether or
/// given once per record. A family that names no records at all is complete,
/// which is how an omitted optional record stays legal.
///
/// `required` and `optional` hold how many times each flag in the family was
/// given. The number of records is what the required flags agree on, so an
/// incomplete family is one where they disagree, or where an optional flag
/// was given a number of times that fits no record. Each caller phrases its
/// own rejection, because the flag names are what the operator needs to read
/// back.
#[rule("rule_flag_sets_complete")]
pub(super) fn complete_flag_set(required: &[usize], optional: &[usize]) -> bool {
    let records = required.first().copied().unwrap_or(0);
    required.iter().all(|&count| count == records)
        && optional.iter().all(|&count| count == 0 || count == records)
}

pub(super) fn parse_json_arg<T: DeserializeOwned>(flag: &str, value: &str) -> anyhow::Result<T> {
    let json = if let Some(path) = value.strip_prefix('@') {
        Cow::Owned(
            std::fs::read_to_string(path)
                .with_context(|| format!("failed to read --{flag} JSON file {path}"))?,
        )
    } else {
        Cow::Borrowed(value)
    };
    serde_json::from_str(&json).with_context(|| format!("--{flag} must be valid JSON"))
}

pub(super) fn stable_ids(values: Vec<String>) -> anyhow::Result<Vec<StableId>> {
    values.into_iter().map(StableId::new).collect()
}

pub(super) fn resolution_inputs(
    input_types: Vec<String>,
    references: Vec<String>,
    summaries: Vec<String>,
) -> anyhow::Result<Vec<ResolutionInput>> {
    anyhow::ensure!(
        complete_flag_set(&[input_types.len(), references.len(), summaries.len()], &[]),
        "--input-type, --input-reference, and --input-summary must be provided the same number of times"
    );
    input_types
        .into_iter()
        .zip(references)
        .zip(summaries)
        .map(|((input_type, reference), summary)| {
            validate_resolution_input_content(&reference, &summary)?;
            Ok(ResolutionInput {
                input_type: ResolutionInputType::parse(&input_type)?,
                reference,
                summary,
            })
        })
        .collect()
}

pub(super) fn ideation_target(
    target_type: &str,
    target_id: String,
) -> anyhow::Result<IdeationTarget> {
    Ok(IdeationTarget {
        artifact_type: IdeationTargetType::parse(target_type)?,
        artifact_id: StableId::new(target_id)?,
    })
}

pub(super) fn canonical_artifact(
    artifact_type: Option<String>,
    artifact_id: Option<String>,
) -> anyhow::Result<Option<CanonicalArtifact>> {
    anyhow::ensure!(
        complete_flag_set(
            &[
                usize::from(artifact_type.is_some()),
                usize::from(artifact_id.is_some()),
            ],
            &[],
        ),
        "--canonical-artifact-type and --canonical-artifact-id must be provided together"
    );
    artifact_type
        .zip(artifact_id)
        .map(|(artifact_type, artifact_id)| {
            Ok(CanonicalArtifact {
                artifact_type: CanonicalArtifactType::parse(&artifact_type)?,
                artifact_id: StableId::new(artifact_id)?,
            })
        })
        .transpose()
}

pub(super) fn warn_if_skills_missing(repo: &Utf8PathBuf, quiet: bool) -> anyhow::Result<()> {
    if quiet {
        return Ok(());
    }
    let status = skills::install_status(repo.as_std_path())?;
    if !status.installed {
        eprintln!(
            "hint: provenance skills are not installed; run `{}` from the repo root",
            status.install_command
        );
    }
    Ok(())
}

pub(super) fn boundary_source_ref(
    source_id: Option<String>,
    source_clause: Option<String>,
) -> anyhow::Result<Option<SourceReference>> {
    anyhow::ensure!(
        complete_flag_set(
            &[usize::from(source_id.is_some())],
            &[usize::from(source_clause.is_some())],
        ),
        "--source-clause requires --source-id"
    );
    source_id
        .map(|source_id| {
            Ok(SourceReference {
                source_id: StableId::new(source_id)?,
                clause: source_clause,
            })
        })
        .transpose()
}

#[cfg(test)]
mod tests;
