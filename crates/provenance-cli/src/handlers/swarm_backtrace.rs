use super::validate::{
    validate_contribution_record, validate_proposal_card_record, validate_synthesis_packet_record,
};
use crate::{
    cli::SwarmBacktraceCommand,
    output::{self, OutputFormat},
};
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_core::{Contribution, ProposalCard, ScopeId, StableId, SynthesisPacket};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{
        CreateContributionInput, CreateProposalCardInput, CreateSynthesisPacketInput, StateStore,
    },
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
    replace: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ContributionFile {
    Wrapped { contribution: Contribution },
    Many { contributions: Vec<Contribution> },
    Direct(Contribution),
    DirectMany(Vec<Contribution>),
}

impl ContributionFile {
    fn into_records(self) -> Vec<Contribution> {
        match self {
            Self::Wrapped { contribution } | Self::Direct(contribution) => vec![contribution],
            Self::Many { contributions } | Self::DirectMany(contributions) => contributions,
        }
    }
}

#[derive(Deserialize)]
struct MergeOutput {
    #[serde(default)]
    synthesis_packet: Option<SynthesisPacket>,
    #[serde(default, alias = "synthesis")]
    synthesis_packets: Vec<SynthesisPacket>,
    #[serde(default, alias = "proposal_cards")]
    proposals: Vec<ProposalCard>,
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
    let (synthesis_packets, proposals) = read_merge_outputs(run_dir)?;

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

    let contribution_count = contributions.len();
    let synthesis_count = synthesis_packets.len();
    let proposal_count = proposals.len();
    let store = StateStore::new(ProvenanceLayout::new(repo));

    for contribution in contributions {
        let input = contribution_input(&scope_id, contribution);
        if replace {
            store.upsert_contribution(input)?;
        } else {
            store.create_contribution(input)?;
        }
    }
    for synthesis_packet in synthesis_packets {
        let input = synthesis_packet_input(&scope_id, synthesis_packet);
        if replace {
            store.upsert_synthesis_packet(input)?;
        } else {
            store.create_synthesis_packet(input)?;
        }
    }
    for proposal in proposals {
        let input = proposal_input(&scope_id, proposal);
        if replace {
            store.upsert_proposal_card(input)?;
        } else {
            store.create_proposal_card(input)?;
        }
    }

    output::print(
        format,
        &LandReport {
            run_dir: run_dir.to_string(),
            contributions: contribution_count,
            synthesis_packets: synthesis_count,
            proposals: proposal_count,
            replace,
        },
    )
}

fn read_contributions(run_dir: &Utf8Path) -> anyhow::Result<Vec<Contribution>> {
    let mut contributions = Vec::new();
    for directory in ["extractors", "refuters", "contributions"] {
        for path in json_files(&run_dir.join(directory))? {
            let file: ContributionFile = read_json(&path)?;
            contributions.extend(file.into_records());
        }
    }
    Ok(contributions)
}

fn read_merge_outputs(
    run_dir: &Utf8Path,
) -> anyhow::Result<(Vec<SynthesisPacket>, Vec<ProposalCard>)> {
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
    for path in paths {
        let merge_output: MergeOutput = read_json(&path)?;
        if let Some(synthesis_packet) = merge_output.synthesis_packet {
            synthesis_packets.push(synthesis_packet);
        }
        synthesis_packets.extend(merge_output.synthesis_packets);
        proposals.extend(merge_output.proposals);
    }
    Ok((synthesis_packets, proposals))
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

fn contribution_input(scope_id: &ScopeId, contribution: Contribution) -> CreateContributionInput {
    let Contribution {
        id,
        target,
        participant_slot,
        stance,
        strongest_finding,
        evidence_references,
        material_claims,
        risks,
        objections,
        challenges,
        suggested_artifact_changes,
        unsupported_recommendations,
        uncertainty,
        open_questions,
        ..
    } = contribution;
    CreateContributionInput {
        scope_id: scope_id.clone(),
        id,
        target,
        participant_slot,
        stance,
        strongest_finding,
        evidence_references,
        material_claims,
        risks,
        objections,
        challenges,
        suggested_artifact_changes,
        unsupported_recommendations,
        uncertainty,
        open_questions,
    }
}

fn synthesis_packet_input(
    scope_id: &ScopeId,
    synthesis_packet: SynthesisPacket,
) -> CreateSynthesisPacketInput {
    let SynthesisPacket {
        id,
        target,
        summary,
        consensus,
        contested_claims,
        minority_objections,
        evidence_gaps,
        unsupported_speculation,
        open_questions,
        suggested_artifacts,
        required_human_decisions,
        ..
    } = synthesis_packet;
    CreateSynthesisPacketInput {
        scope_id: scope_id.clone(),
        id,
        target,
        summary,
        consensus,
        contested_claims,
        minority_objections,
        evidence_gaps,
        unsupported_speculation,
        open_questions,
        suggested_artifacts,
        required_human_decisions,
    }
}

fn proposal_input(scope_id: &ScopeId, proposal: ProposalCard) -> CreateProposalCardInput {
    let ProposalCard {
        id,
        proposal_key,
        proposal_type,
        title,
        summary,
        confidence,
        traceability,
        promotion_state,
        duplicate_of,
        superseded_by,
        ..
    } = proposal;
    CreateProposalCardInput {
        scope_id: scope_id.clone(),
        id,
        proposal_key,
        proposal_type,
        title,
        summary,
        confidence,
        traceability,
        promotion_state,
        duplicate_of,
        superseded_by,
    }
}
