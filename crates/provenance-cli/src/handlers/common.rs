use crate::skills;
use anyhow::Context;
use camino::Utf8PathBuf;
use provenance_core::{
    CanonicalArtifact, CanonicalArtifactType, IdeationTarget, IdeationTargetType, ResolutionInput,
    ResolutionInputType, SourceReference, StableId,
};
use serde::de::DeserializeOwned;
use std::borrow::Cow;

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
        input_types.len() == references.len() && references.len() == summaries.len(),
        "--input-type, --input-reference, and --input-summary must be provided the same number of times"
    );
    input_types
        .into_iter()
        .zip(references)
        .zip(summaries)
        .map(|((input_type, reference), summary)| {
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
    match (artifact_type, artifact_id) {
        (Some(artifact_type), Some(artifact_id)) => Ok(Some(CanonicalArtifact {
            artifact_type: CanonicalArtifactType::parse(&artifact_type)?,
            artifact_id: StableId::new(artifact_id)?,
        })),
        (None, None) => Ok(None),
        _ => anyhow::bail!(
            "--canonical-artifact-type and --canonical-artifact-id must be provided together"
        ),
    }
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
    match (source_id, source_clause) {
        (Some(source_id), source_clause) => Ok(Some(SourceReference {
            source_id: StableId::new(source_id)?,
            clause: source_clause,
        })),
        (None, None) => Ok(None),
        (None, Some(_)) => anyhow::bail!("--source-clause requires --source-id"),
    }
}
