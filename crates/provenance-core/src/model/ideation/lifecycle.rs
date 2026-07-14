use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use super::{IdentityType, PromotionState, ProposalType};
use crate::model::{
    validate_optional_confidence_score, Contribution, DispositionRecord, ProposalCard,
    SchemaVersion, ScopeId, StableId, SynthesisPacket,
};

/// Identity of immutable positive adjudication evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssertionId(StableId);

impl AssertionId {
    pub fn new(value: impl Into<String>) -> anyhow::Result<Self> {
        StableId::new(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub const fn as_stable_id(&self) -> &StableId {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionRecord {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: AssertionId,
    pub proposal_id: StableId,
    pub synthesis_packet_id: StableId,
    pub supporting_claim_ids: Vec<StableId>,
}

#[derive(Clone, Copy)]
pub struct IdeationAggregate<'a> {
    pub contributions: &'a [Contribution],
    pub synthesis_packets: &'a [SynthesisPacket],
    pub proposals: &'a [ProposalCard],
    pub assertions: &'a [AssertionRecord],
    pub dispositions: &'a [DispositionRecord],
}

pub fn validate_proposal_intrinsic(proposal: &ProposalCard) -> anyhow::Result<()> {
    validate_proposal_common(proposal)?;
    anyhow::ensure!(
        proposal.promotion_state == PromotionState::Proposed,
        "proposals must begin proposed; assertion and disposition are verified transitions"
    );
    anyhow::ensure!(
        !proposal.legacy_terminal,
        "active proposals cannot claim legacy terminal compatibility"
    );
    anyhow::ensure!(
        proposal.duplicate_of.is_none() && proposal.superseded_by.is_none(),
        "proposal disposition links require an authoritative disposition record"
    );
    ensure_unique_lineage(proposal)
}

fn validate_proposal_common(proposal: &ProposalCard) -> anyhow::Result<()> {
    anyhow::ensure!(
        proposal.schema_version.0 == 1,
        "proposal schema_version must be 1"
    );
    anyhow::ensure!(
        !proposal.proposal_key.trim().is_empty(),
        "proposal_key must not be empty"
    );
    validate_optional_confidence_score(proposal.confidence)?;
    Ok(())
}

fn ensure_unique_lineage(proposal: &ProposalCard) -> anyhow::Result<()> {
    let mut lineage = BTreeSet::new();
    for assertion_id in &proposal.builds_on {
        anyhow::ensure!(
            lineage.insert(assertion_id),
            "builds_on assertion {} is repeated",
            assertion_id.as_str()
        );
    }
    Ok(())
}

pub fn validate_ideation_aggregate(aggregate: IdeationAggregate<'_>) -> anyhow::Result<()> {
    ensure_unique(
        "contribution",
        aggregate
            .contributions
            .iter()
            .map(|record| record.id.as_str()),
    )?;
    ensure_unique(
        "synthesis packet",
        aggregate
            .synthesis_packets
            .iter()
            .map(|record| record.id.as_str()),
    )?;
    for contribution in aggregate.contributions {
        anyhow::ensure!(
            contribution.schema_version.0 == 1,
            "contribution schema_version must be 1"
        );
        for claim in &contribution.material_claims {
            validate_optional_confidence_score(claim.confidence)?;
        }
    }
    for packet in aggregate.synthesis_packets {
        anyhow::ensure!(
            packet.schema_version.0 == 1,
            "synthesis packet schema_version must be 1"
        );
        ensure_unique(
            "suggested proposal",
            packet
                .suggested_artifacts
                .iter()
                .filter_map(|suggestion| suggestion.proposal_id.as_ref().map(StableId::as_str)),
        )?;
    }
    ensure_unique(
        "proposal",
        aggregate.proposals.iter().map(|p| p.id.as_str()),
    )?;
    ensure_unique(
        "assertion",
        aggregate.assertions.iter().map(|a| a.id.as_str()),
    )?;
    for proposal in aggregate.proposals {
        validate_proposal_common(proposal)?;
        if proposal.promotion_state == PromotionState::Proposed {
            anyhow::ensure!(
                proposal.duplicate_of.is_none() && proposal.superseded_by.is_none(),
                "proposal disposition links require an authoritative disposition record"
            );
            ensure_unique_lineage(proposal)?;
        } else {
            let matching_historical_disposition = aggregate.dispositions.iter().any(|record| {
                record.proposal_id == proposal.id
                    && disposition_matches_state(record.decision, proposal.promotion_state)
            });
            anyhow::ensure!(
                (proposal.legacy_terminal || matching_historical_disposition)
                    && matches!(
                        proposal.promotion_state,
                        PromotionState::Accepted
                            | PromotionState::Rejected
                            | PromotionState::Deferred
                            | PromotionState::Duplicate
                            | PromotionState::Superseded
                    ),
                "embedded state is not a safely identifiable legacy terminal record"
            );
        }
    }

    let proposals = aggregate
        .proposals
        .iter()
        .map(|proposal| (proposal.id.as_str(), proposal))
        .collect::<BTreeMap<_, _>>();
    let synthesis = aggregate
        .synthesis_packets
        .iter()
        .map(|packet| (packet.id.as_str(), packet))
        .collect::<BTreeMap<_, _>>();
    let assertions = aggregate
        .assertions
        .iter()
        .map(|assertion| (assertion.id.as_str(), assertion))
        .collect::<BTreeMap<_, _>>();
    ensure_assertions(&aggregate, &proposals, &synthesis)?;
    ensure_dispositions(&aggregate, &proposals, &assertions)?;
    let current_proposals = aggregate
        .proposals
        .iter()
        .filter(|proposal| proposal.promotion_state == PromotionState::Proposed)
        .cloned()
        .collect::<Vec<_>>();
    ensure_lineage(&current_proposals, &assertions)
}

#[allow(clippy::too_many_lines)]
fn ensure_assertions<'a>(
    aggregate: &IdeationAggregate<'a>,
    proposals: &BTreeMap<&str, &'a ProposalCard>,
    synthesis: &BTreeMap<&str, &'a SynthesisPacket>,
) -> anyhow::Result<()> {
    let mut asserted_proposals = BTreeSet::new();
    let mut claims = BTreeMap::new();
    for contribution in aggregate.contributions {
        for claim in &contribution.material_claims {
            claims
                .entry(claim.claim_id.as_str())
                .or_insert_with(Vec::new)
                .push((claim, contribution));
        }
    }
    for assertion in aggregate.assertions {
        anyhow::ensure!(
            assertion.schema_version.0 == 1,
            "assertion schema_version must be 1"
        );
        let proposal = proposals
            .get(assertion.proposal_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "asserted proposal {} does not exist",
                    assertion.proposal_id.as_str()
                )
            })?;
        anyhow::ensure!(
            proposal.promotion_state == PromotionState::Proposed,
            "legacy terminal proposal {} is frozen and cannot be asserted",
            proposal.id.as_str()
        );
        anyhow::ensure!(
            asserted_proposals.insert(assertion.proposal_id.as_str()),
            "proposal {} has multiple assertions",
            assertion.proposal_id.as_str()
        );
        let packet = synthesis
            .get(assertion.synthesis_packet_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "assertion synthesis packet {} does not exist",
                    assertion.synthesis_packet_id.as_str()
                )
            })?;
        anyhow::ensure!(
            packet.target == proposal.traceability.target,
            "assertion synthesis is not owned by the proposal target"
        );
        anyhow::ensure!(
            assertion.scope_id == proposal.scope_id && packet.scope_id == proposal.scope_id,
            "assertion records must share the proposal scope"
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
            !assertion.supporting_claim_ids.is_empty(),
            "assertion requires positive evidence"
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
            let owners = claims.get(claim_id.as_str()).ok_or_else(|| {
                anyhow::anyhow!("assertion claim {} does not exist", claim_id.as_str())
            })?;
            anyhow::ensure!(
                owners.len() == 1,
                "assertion claim {} has multiple owners",
                claim_id.as_str()
            );
            let (claim, owner) = owners[0];
            anyhow::ensure!(
                claim.evidence_type.is_positive(),
                "assertion claim {} must use a positive evidence type",
                claim_id.as_str()
            );
            anyhow::ensure!(
                owner.target == proposal.traceability.target,
                "assertion claim {} is not owned by the proposal target",
                claim_id.as_str()
            );
            anyhow::ensure!(
                owner.scope_id == proposal.scope_id,
                "assertion claim {} is not owned by the proposal scope",
                claim_id.as_str()
            );
            anyhow::ensure!(
                !claim.evidence_reference_ids.is_empty(),
                "assertion claim {} lacks positive evidence",
                claim_id.as_str()
            );
            for evidence_id in &claim.evidence_reference_ids {
                let evidence = owner
                    .evidence_references
                    .iter()
                    .find(|evidence| evidence.reference_id == *evidence_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "assertion evidence {} does not exist in the claim owner",
                            evidence_id.as_str()
                        )
                    })?;
                anyhow::ensure!(
                    evidence.evidence_type == claim.evidence_type,
                    "assertion evidence type does not match claim {}",
                    claim_id.as_str()
                );
                anyhow::ensure!(
                    evidence.evidence_type.is_positive(),
                    "assertion evidence {} must use a positive evidence type",
                    evidence_id.as_str()
                );
            }
        }
        anyhow::ensure!(
            assertion.supporting_claim_ids == proposal.traceability.supporting_claim_ids,
            "assertion claims must match proposal traceability"
        );
    }
    Ok(())
}

fn ensure_dispositions(
    aggregate: &IdeationAggregate<'_>,
    proposals: &BTreeMap<&str, &ProposalCard>,
    assertions: &BTreeMap<&str, &AssertionRecord>,
) -> anyhow::Result<()> {
    let mut disposed = BTreeSet::new();
    ensure_unique(
        "disposition",
        aggregate.dispositions.iter().map(|d| d.id.as_str()),
    )?;
    for disposition in aggregate.dispositions {
        anyhow::ensure!(
            disposition.schema_version.0 == 1,
            "disposition schema_version must be 1"
        );
        let proposal = proposals
            .get(disposition.proposal_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "disposition proposal {} does not exist",
                    disposition.proposal_id.as_str()
                )
            })?;
        anyhow::ensure!(
            disposition.scope_id == proposal.scope_id,
            "disposition must share the proposal scope"
        );
        if proposal.promotion_state != PromotionState::Proposed {
            anyhow::ensure!(
                !assertions
                    .values()
                    .any(|assertion| assertion.proposal_id == disposition.proposal_id),
                "legacy terminal proposal {} is frozen against assertions",
                proposal.id.as_str()
            );
            anyhow::ensure!(
                disposition_matches_state(disposition.decision, proposal.promotion_state),
                "legacy terminal proposal state disagrees with its historical disposition"
            );
            anyhow::ensure!(
                disposed.insert(disposition.proposal_id.as_str()),
                "proposal {} has contradictory dispositions",
                disposition.proposal_id.as_str()
            );
            continue;
        }
        anyhow::ensure!(
            assertions
                .values()
                .any(|assertion| assertion.proposal_id == disposition.proposal_id),
            "proposal {} must be asserted before disposition",
            disposition.proposal_id.as_str()
        );
        anyhow::ensure!(
            disposed.insert(disposition.proposal_id.as_str()),
            "proposal {} has contradictory dispositions",
            disposition.proposal_id.as_str()
        );
        ensure_authoritative_actor(proposal, disposition.actor.identity_type)?;
    }
    Ok(())
}

const fn disposition_matches_state(
    decision: crate::model::PromotionDecision,
    state: PromotionState,
) -> bool {
    matches!(
        (decision, state),
        (
            crate::model::PromotionDecision::Accepted,
            PromotionState::Accepted
        ) | (
            crate::model::PromotionDecision::Rejected,
            PromotionState::Rejected
        ) | (
            crate::model::PromotionDecision::Deferred,
            PromotionState::Deferred
        )
    )
}

pub fn ensure_authoritative_actor(
    proposal: &ProposalCard,
    identity_type: IdentityType,
) -> anyhow::Result<()> {
    let changes_behavior = matches!(
        proposal.proposal_type,
        ProposalType::RequirementCandidate
            | ProposalType::ResolutionCandidate
            | ProposalType::RuleCandidate
    );
    anyhow::ensure!(
        !changes_behavior || identity_type == IdentityType::Human,
        "authoritative disposition requires a human actor"
    );
    Ok(())
}

fn ensure_lineage(
    proposals: &[ProposalCard],
    assertions: &BTreeMap<&str, &AssertionRecord>,
) -> anyhow::Result<()> {
    let proposal_edges = proposals
        .iter()
        .map(|proposal| {
            let ancestors = proposal
                .builds_on
                .iter()
                .map(|id| {
                    assertions
                        .get(id.as_str())
                        .map(|a| a.proposal_id.as_str())
                        .ok_or_else(|| {
                            anyhow::anyhow!("builds_on assertion {} does not exist", id.as_str())
                        })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            Ok((proposal.id.as_str(), ancestors))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for proposal in proposal_edges.keys() {
        visit(proposal, &proposal_edges, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit<'a>(
    proposal: &'a str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> anyhow::Result<()> {
    if visited.contains(proposal) {
        return Ok(());
    }
    anyhow::ensure!(
        visiting.insert(proposal),
        "proposal assertion lineage contains a cycle at {proposal}"
    );
    for ancestor in edges.get(proposal).into_iter().flatten() {
        visit(ancestor, edges, visiting, visited)?;
    }
    visiting.remove(proposal);
    visited.insert(proposal);
    Ok(())
}

fn ensure_unique<'a>(kind: &str, ids: impl IntoIterator<Item = &'a str>) -> anyhow::Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        anyhow::ensure!(seen.insert(id), "duplicate {kind} id {id}");
    }
    Ok(())
}
