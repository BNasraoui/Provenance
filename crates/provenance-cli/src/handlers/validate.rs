use crate::{
    cli::IdeationArtifactKind,
    output::{self, OutputFormat},
};
use camino::Utf8PathBuf;
use provenance_core::{Contribution, PromotionState, ProposalCard, SchemaVersion, SynthesisPacket};
use serde::Serialize;

#[derive(Serialize)]
struct ValidationReport {
    artifact: &'static str,
    input: String,
    valid: bool,
}

pub(super) fn handle(
    artifact: IdeationArtifactKind,
    input: Utf8PathBuf,
    format: OutputFormat,
) -> anyhow::Result<()> {
    validate_file(artifact, &input)?;
    output::print(
        format,
        &ValidationReport {
            artifact: artifact.name(),
            input: input.to_string(),
            valid: true,
        },
    )
}

pub(super) fn validate_file(
    artifact: IdeationArtifactKind,
    input: &Utf8PathBuf,
) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(input)?;
    match artifact {
        IdeationArtifactKind::Contribution => {
            let contribution: Contribution = serde_json::from_str(&json)?;
            ensure_schema_version(contribution.schema_version)?;
        }
        IdeationArtifactKind::SynthesisPacket => {
            let synthesis_packet: SynthesisPacket = serde_json::from_str(&json)?;
            ensure_schema_version(synthesis_packet.schema_version)?;
        }
        IdeationArtifactKind::Proposal => {
            let proposal: ProposalCard = serde_json::from_str(&json)?;
            ensure_schema_version(proposal.schema_version)?;
            validate_proposal_card(&proposal)?;
        }
    }
    Ok(())
}

fn ensure_schema_version(schema_version: SchemaVersion) -> anyhow::Result<()> {
    anyhow::ensure!(
        schema_version == SchemaVersion(1),
        "schema_version must be 1"
    );
    Ok(())
}

pub(super) fn validate_proposal_card(proposal: &ProposalCard) -> anyhow::Result<()> {
    match proposal.promotion_state {
        PromotionState::Duplicate => {
            anyhow::ensure!(
                proposal.duplicate_of.is_some(),
                "duplicate proposals must set duplicate_of"
            );
        }
        PromotionState::Superseded => {
            anyhow::ensure!(
                proposal.superseded_by.is_some(),
                "superseded proposals must set superseded_by"
            );
        }
        PromotionState::Proposed
        | PromotionState::Accepted
        | PromotionState::Rejected
        | PromotionState::Deferred => {}
    }
    Ok(())
}
