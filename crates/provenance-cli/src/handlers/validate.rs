use crate::{
    cli::IdeationArtifactKind,
    output::{self, OutputFormat},
};
use camino::Utf8Path;
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
    input: &Utf8Path,
    format: OutputFormat,
) -> anyhow::Result<()> {
    validate_file(artifact, input)?;
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
    input: &Utf8Path,
) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(input)?;
    match artifact {
        IdeationArtifactKind::Contribution => {
            let contribution: Contribution = serde_json::from_str(&json)?;
            validate_contribution_record(&contribution)?;
        }
        IdeationArtifactKind::SynthesisPacket => {
            let synthesis_packet: SynthesisPacket = serde_json::from_str(&json)?;
            validate_synthesis_packet_record(&synthesis_packet)?;
        }
        IdeationArtifactKind::Proposal => {
            let proposal: ProposalCard = serde_json::from_str(&json)?;
            validate_proposal_card_record(&proposal)?;
        }
    }
    Ok(())
}

pub(super) fn validate_contribution_record(contribution: &Contribution) -> anyhow::Result<()> {
    ensure_schema_version(contribution.schema_version)
}

pub(super) fn validate_synthesis_packet_record(
    synthesis_packet: &SynthesisPacket,
) -> anyhow::Result<()> {
    ensure_schema_version(synthesis_packet.schema_version)
}

fn ensure_schema_version(schema_version: SchemaVersion) -> anyhow::Result<()> {
    anyhow::ensure!(
        schema_version == SchemaVersion(1),
        "schema_version must be 1"
    );
    Ok(())
}

pub(super) fn validate_proposal_card_record(proposal: &ProposalCard) -> anyhow::Result<()> {
    ensure_schema_version(proposal.schema_version)?;
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
