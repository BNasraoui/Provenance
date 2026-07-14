use super::validate::{
    validate_contribution_record, validate_proposal_card_record, validate_synthesis_packet_record,
};
use crate::{
    cli::ideation::SwarmBacktraceCommand,
    output::{self, OutputFormat},
};
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_core::{
    AssertionRecord, Contribution, ProposalCard, ScopeId, StableId, SynthesisPacket,
};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{IdeationLandingBatch, StateStore},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

pub(super) fn handle(command: SwarmBacktraceCommand) -> anyhow::Result<()> {
    match command {
        SwarmBacktraceCommand::Land {
            repo,
            scope,
            run_dir,
            replace,
            format,
        } => land(repo, scope, &run_dir, replace, format),
    }
}

#[derive(Serialize)]
struct LandReport {
    run_dir: String,
    contributions: usize,
    synthesis_packets: usize,
    proposals: usize,
    assertions: usize,
    replace: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MergeOutput {
    #[serde(default)]
    synthesis_packet: Option<SynthesisPacket>,
    #[serde(default, alias = "synthesis")]
    synthesis_packets: Vec<SynthesisPacket>,
    #[serde(default, alias = "proposal_cards")]
    proposals: Vec<ProposalCard>,
    #[serde(default)]
    assertions: Vec<AssertionRecord>,
}

fn land(
    repo: Utf8PathBuf,
    scope: String,
    run_dir: &Utf8Path,
    replace: bool,
    format: OutputFormat,
) -> anyhow::Result<()> {
    anyhow::ensure!(run_dir.is_dir(), "--run-dir must be an existing directory");
    let scope_id = ScopeId::new(scope)?;
    let contributions = read_contributions(run_dir)?;
    let (synthesis_packets, proposals, assertions) = read_merge_outputs(run_dir)?;

    anyhow::ensure!(
        !contributions.is_empty(),
        "no contribution JSON files found under extractors/, refuters/, or contributions/"
    );
    anyhow::ensure!(
        !synthesis_packets.is_empty(),
        "no synthesis packet found in merge output"
    );

    for contribution in &contributions {
        validate_contribution_record(contribution)?;
        ensure_scope(
            &scope_id,
            &contribution.scope_id,
            "contribution",
            &contribution.id,
        )?;
    }
    for synthesis_packet in &synthesis_packets {
        validate_synthesis_packet_record(synthesis_packet)?;
        ensure_scope(
            &scope_id,
            &synthesis_packet.scope_id,
            "synthesis packet",
            &synthesis_packet.id,
        )?;
    }
    for proposal in &proposals {
        validate_proposal_card_record(proposal)?;
        ensure_scope(&scope_id, &proposal.scope_id, "proposal", &proposal.id)?;
    }
    for assertion in &assertions {
        ensure_scope(
            &scope_id,
            &assertion.scope_id,
            "assertion",
            &StableId::new(assertion.id.as_str())?,
        )?;
    }

    let contribution_count = contributions.len();
    let synthesis_count = synthesis_packets.len();
    let proposal_count = proposals.len();
    let assertion_count = assertions.len();
    let store = StateStore::new(ProvenanceLayout::new(repo));
    store.land_ideation_batch(
        &scope_id,
        IdeationLandingBatch {
            contributions,
            synthesis_packets,
            proposals,
            assertions,
            dispositions: Vec::new(),
        },
        replace,
    )?;

    output::print(
        format,
        &LandReport {
            run_dir: run_dir.to_string(),
            contributions: contribution_count,
            synthesis_packets: synthesis_count,
            proposals: proposal_count,
            assertions: assertion_count,
            replace,
        },
    )
}

fn read_contributions(run_dir: &Utf8Path) -> anyhow::Result<Vec<Contribution>> {
    let mut contributions = Vec::new();
    for directory in ["extractors", "refuters", "contributions"] {
        for path in json_files(&run_dir.join(directory))? {
            contributions.extend(read_contribution_file(&path)?);
        }
    }
    Ok(contributions)
}

fn read_contribution_file(path: &Utf8Path) -> anyhow::Result<Vec<Contribution>> {
    let value: Value = read_json(path)?;
    if let Some(contribution) = value.get("contribution") {
        return deserialize_landing_value(path, "contribution", contribution.clone())
            .map(|contribution| vec![contribution]);
    }
    if let Some(contributions) = value.get("contributions") {
        return deserialize_landing_value(path, "contributions", contributions.clone());
    }
    if value.is_array() {
        return deserialize_landing_value(path, "contributions", value);
    }
    deserialize_landing_value(path, "contribution", value).map(|contribution| vec![contribution])
}

fn read_merge_outputs(
    run_dir: &Utf8Path,
) -> anyhow::Result<(
    Vec<SynthesisPacket>,
    Vec<ProposalCard>,
    Vec<AssertionRecord>,
)> {
    let mut paths = json_files(&run_dir.join("merge"))?;
    for file_name in ["merged.json", "merge.json"] {
        let path = run_dir.join(file_name);
        if path.exists() {
            paths.push(path);
        }
    }
    paths.sort();
    paths.dedup();

    let mut synthesis_packets = Vec::new();
    let mut proposals = Vec::new();
    let mut assertions = Vec::new();
    for path in paths {
        let merge_output: MergeOutput = read_json(&path)?;
        if let Some(synthesis_packet) = merge_output.synthesis_packet {
            synthesis_packets.push(synthesis_packet);
        }
        synthesis_packets.extend(merge_output.synthesis_packets);
        proposals.extend(merge_output.proposals);
        assertions.extend(merge_output.assertions);
    }
    Ok((synthesis_packets, proposals, assertions))
}

fn json_files(directory: &Utf8Path) -> anyhow::Result<Vec<Utf8PathBuf>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in
        std::fs::read_dir(directory).with_context(|| format!("failed to read {directory}"))?
    {
        let path = utf8_path(entry?.path())?;
        if path.extension() == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn read_json<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<T> {
    let json = std::fs::read_to_string(path).with_context(|| format!("failed to read {path}"))?;
    serde_json::from_str(&json).with_context(|| format!("{path} must be valid landing JSON"))
}

fn deserialize_landing_value<T: DeserializeOwned>(
    path: &Utf8Path,
    field: &str,
    value: Value,
) -> anyhow::Result<T> {
    serde_json::from_value(value)
        .with_context(|| format!("{path} {field} must be valid landing JSON"))
}

fn utf8_path(path: std::path::PathBuf) -> anyhow::Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path)
        .map_err(|path| anyhow::anyhow!("path is not valid UTF-8: {}", path.display()))
}

fn ensure_scope(
    expected: &ScopeId,
    actual: &ScopeId,
    artifact: &str,
    id: &StableId,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        actual == expected,
        "{artifact} {} scope_id must match --scope {}",
        id.as_str(),
        expected.as_str()
    );
    Ok(())
}
