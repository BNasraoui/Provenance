use super::{validate_assertion_intrinsic, AssertionRecord, IdeationAggregate};
use crate::model::{
    Contribution, IdeationEvidenceType, MaterialClaim, PromotionState, ProposalCard, StableId,
    SynthesisPacket,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate_assertions(
    aggregate: &IdeationAggregate<'_>,
    proposals: &BTreeMap<&str, &ProposalCard>,
    synthesis_packets: &BTreeMap<&str, &SynthesisPacket>,
) -> anyhow::Result<()> {
    let mut asserted_proposals = BTreeSet::new();
    for assertion in aggregate.assertions {
        anyhow::ensure!(
            assertion.schema_version.0 == 1,
            "assertion schema_version must be 1"
        );
        validate_assertion_intrinsic(assertion)?;
        let proposal = proposals
            .get(assertion.proposal_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("asserted proposal does not exist"))?;
        validate_assertion_proposal(assertion, proposal, &mut asserted_proposals)?;
        let packet = synthesis_packets
            .get(assertion.synthesis_packet_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("assertion synthesis packet does not exist"))?;
        validate_assertion_packet(proposal, packet)?;
        validate_supporting_claims(assertion, proposal, packet, aggregate.contributions)?;
        let assertion_claims = assertion
            .supporting_claim_ids
            .iter()
            .map(StableId::as_str)
            .collect::<BTreeSet<_>>();
        let proposal_claims = proposal
            .traceability
            .supporting_claim_ids
            .iter()
            .map(StableId::as_str)
            .collect::<BTreeSet<_>>();
        anyhow::ensure!(
            assertion_claims == proposal_claims,
            "assertion claims must match proposal traceability"
        );
    }
    Ok(())
}

fn validate_assertion_proposal<'a>(
    assertion: &'a AssertionRecord,
    proposal: &ProposalCard,
    asserted_proposals: &mut BTreeSet<&'a str>,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        proposal.promotion_state == PromotionState::Proposed,
        "legacy terminal proposal {} is frozen against lifecycle re-entry",
        proposal.id.as_str()
    );
    anyhow::ensure!(
        assertion.scope_id == proposal.scope_id,
        "assertion must share the proposal scope"
    );
    anyhow::ensure!(
        asserted_proposals.insert(assertion.proposal_id.as_str()),
        "proposal {} has multiple assertions",
        assertion.proposal_id.as_str()
    );
    Ok(())
}

fn validate_assertion_packet(
    proposal: &ProposalCard,
    packet: &SynthesisPacket,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        packet.scope_id == proposal.scope_id && packet.target == proposal.traceability.target,
        "assertion synthesis is not owned by the proposal target"
    );
    anyhow::ensure!(
        packet.suggested_artifacts.iter().any(|suggestion| {
            suggestion.proposal_id.as_ref() == Some(&proposal.id)
                && suggestion.proposal_key == proposal.proposal_key
                && suggestion.proposal_type == proposal.proposal_type
        }),
        "synthesis packet does not adjudicate proposal {}",
        proposal.id.as_str()
    );
    anyhow::ensure!(
        !packet
            .evidence_gaps
            .iter()
            .any(|gap| gap.blocking_promotion),
        "assertion has a blocking evidence gap"
    );
    anyhow::ensure!(
        !packet
            .required_human_decisions
            .iter()
            .any(|decision| decision.blocks_promotion),
        "assertion has a blocking human decision"
    );
    Ok(())
}

fn validate_supporting_claims(
    assertion: &AssertionRecord,
    proposal: &ProposalCard,
    packet: &SynthesisPacket,
    contributions: &[Contribution],
) -> anyhow::Result<()> {
    let contested = packet
        .contested_claims
        .iter()
        .map(|claim| claim.claim_id.as_str())
        .collect::<BTreeSet<_>>();
    for claim_id in &assertion.supporting_claim_ids {
        anyhow::ensure!(
            !contested.contains(claim_id.as_str()),
            "assertion claim {} is contested",
            claim_id.as_str()
        );
        let (claim, owner) = find_claim_owner(claim_id, contributions)?;
        validate_claim_owner(claim_id, proposal, claim, owner)?;
        validate_claim_evidence(claim_id, claim, owner, contributions)?;
    }
    Ok(())
}

fn find_claim_owner<'a>(
    claim_id: &StableId,
    contributions: &'a [Contribution],
) -> anyhow::Result<(&'a MaterialClaim, &'a Contribution)> {
    let owners = contributions
        .iter()
        .flat_map(|contribution| {
            contribution
                .material_claims
                .iter()
                .map(move |claim| (claim, contribution))
        })
        .filter(|(claim, _)| claim.claim_id == *claim_id)
        .collect::<Vec<_>>();
    anyhow::ensure!(
        owners.len() == 1,
        "assertion claim {} must have exactly one owner",
        claim_id.as_str()
    );
    Ok(owners[0])
}

fn validate_claim_owner(
    claim_id: &StableId,
    proposal: &ProposalCard,
    claim: &MaterialClaim,
    owner: &Contribution,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        owner.scope_id == proposal.scope_id && owner.target == proposal.traceability.target,
        "assertion claim {} is not owned by the proposal target",
        claim_id.as_str()
    );
    anyhow::ensure!(
        !claim.evidence_reference_ids.is_empty(),
        "assertion claim {} lacks positive evidence",
        claim_id.as_str()
    );
    anyhow::ensure!(
        is_positive_evidence_type(claim.evidence_type),
        "assertion claim {} must use a positive evidence type",
        claim_id.as_str()
    );
    Ok(())
}

fn validate_claim_evidence(
    claim_id: &StableId,
    claim: &MaterialClaim,
    owner: &Contribution,
    contributions: &[Contribution],
) -> anyhow::Result<()> {
    for evidence_id in &claim.evidence_reference_ids {
        let matches = contributions
            .iter()
            .flat_map(|contribution| {
                contribution
                    .evidence_references
                    .iter()
                    .map(move |evidence| (evidence, contribution))
            })
            .filter(|(evidence, _)| evidence.reference_id == *evidence_id)
            .collect::<Vec<_>>();
        anyhow::ensure!(
            matches.len() == 1,
            "assertion evidence {} must have exactly one owner",
            evidence_id.as_str()
        );
        let (evidence, evidence_owner) = matches[0];
        anyhow::ensure!(
            std::ptr::eq(evidence_owner, owner),
            "assertion evidence {} is not owned by claim contribution {}",
            evidence_id.as_str(),
            owner.id.as_str()
        );
        anyhow::ensure!(
            evidence.evidence_type == claim.evidence_type,
            "assertion evidence type does not match claim {}",
            claim_id.as_str()
        );
        anyhow::ensure!(
            is_positive_evidence_type(evidence.evidence_type),
            "assertion evidence {} must use a positive evidence type",
            evidence_id.as_str()
        );
    }
    Ok(())
}

const fn is_positive_evidence_type(evidence_type: IdeationEvidenceType) -> bool {
    !matches!(
        evidence_type,
        IdeationEvidenceType::Unsupported | IdeationEvidenceType::Exploratory
    )
}

pub(super) fn ensure_qualifying_assertions(
    proposals: &[ProposalCard],
    synthesis_packets: &[SynthesisPacket],
    assertions: &[AssertionRecord],
) -> anyhow::Result<()> {
    for proposal in proposals {
        let qualifying = synthesis_packets
            .iter()
            .any(|packet| packet_qualifies_proposal(packet, proposal));
        anyhow::ensure!(
            !qualifying
                || assertions
                    .iter()
                    .any(|assertion| assertion.proposal_id == proposal.id),
            "qualifying proposal {} requires an assertion",
            proposal.id.as_str()
        );
    }
    Ok(())
}

fn packet_qualifies_proposal(packet: &SynthesisPacket, proposal: &ProposalCard) -> bool {
    packet.scope_id == proposal.scope_id
        && packet.target == proposal.traceability.target
        && packet.suggested_artifacts.iter().any(|suggestion| {
            suggestion.proposal_id.as_ref() == Some(&proposal.id)
                && suggestion.proposal_key == proposal.proposal_key
                && suggestion.proposal_type == proposal.proposal_type
        })
        && !packet
            .evidence_gaps
            .iter()
            .any(|gap| gap.blocking_promotion)
        && !packet
            .required_human_decisions
            .iter()
            .any(|decision| decision.blocks_promotion)
        && !proposal.traceability.supporting_claim_ids.is_empty()
        && !packet.contested_claims.iter().any(|contested| {
            proposal
                .traceability
                .supporting_claim_ids
                .contains(&contested.claim_id)
        })
}
