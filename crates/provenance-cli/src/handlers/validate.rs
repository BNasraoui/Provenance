use crate::{
    cli::ideation::IdeationArtifactKind,
    output::{self, OutputFormat},
};
use anyhow::Context;
use camino::Utf8Path;
use provenance_core::{
    validate_optional_confidence_score, validate_proposal_intrinsic, Contribution, ProposalCard,
    SchemaVersion, SynthesisPacket,
};
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
    let json = std::fs::read_to_string(input).with_context(|| format!("failed to read {input}"))?;
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
    ensure_schema_version(contribution.schema_version)?;
    for claim in &contribution.material_claims {
        validate_optional_confidence_score(claim.confidence).with_context(|| {
            format!(
                "material claim {} confidence is invalid",
                claim.claim_id.as_str()
            )
        })?;
    }
    Ok(())
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
    validate_proposal_intrinsic(proposal)?;
    validate_optional_confidence_score(proposal.confidence)
        .with_context(|| format!("proposal {} confidence is invalid", proposal.id.as_str()))?;
    Ok(())
}
