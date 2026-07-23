use super::PromotionState;
use crate::model::{
    validate_disposition_intrinsic, validate_optional_confidence_score, Contribution,
    DispositionRecord, ProposalCard, SchemaVersion, ScopeId, StableId, SynthesisPacket,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[serde(deny_unknown_fields)]
pub struct AssertionRecord {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: AssertionId,
    pub proposal_id: StableId,
    pub synthesis_packet_id: StableId,
    pub supporting_claim_ids: Vec<StableId>,
}

pub fn validate_assertion_intrinsic(assertion: &AssertionRecord) -> anyhow::Result<()> {
    anyhow::ensure!(
        !assertion.supporting_claim_ids.is_empty(),
        "assertion requires positive evidence"
    );
    let mut claims = BTreeSet::new();
    for claim_id in &assertion.supporting_claim_ids {
        anyhow::ensure!(
            claims.insert(claim_id.as_str()),
            "assertion supporting claim {} is repeated",
            claim_id.as_str()
        );
    }
    Ok(())
}

pub type Assertion = AssertionRecord;

#[derive(Clone, Copy)]
pub struct IdeationAggregate<'a> {
    pub legacy_policy: LegacyProposalPolicy,
    pub disposition_actor_ids: &'a [String],
    pub contributions: &'a [Contribution],
    pub synthesis_packets: &'a [SynthesisPacket],
    pub proposals: &'a [ProposalCard],
    pub assertions: &'a [AssertionRecord],
    pub dispositions: &'a [DispositionRecord],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyProposalPolicy {
    ModernOnly,
    ShippedV1,
}

const SHIPPED_LEGACY_TERMINAL_DIGEST_V1: &str =
    "f3438033d8dfcab54788c3dad687eca24de9a0278ada120ba1296b2d2b86f843";
const SHIPPED_LEGACY_DISPOSITION_DIGEST_V1: &str =
    "8f25c3f3028b68b8f914efcec3a19e08127a86797b8a01bcd5f796dac90c9cfb";

pub fn validate_proposal_intrinsic(proposal: &ProposalCard) -> anyhow::Result<()> {
    anyhow::ensure!(
        proposal.promotion_state == PromotionState::Proposed,
        "modern proposals must begin proposed; assertion and disposition derive later state"
    );
    anyhow::ensure!(
        proposal.duplicate_of.is_none() && proposal.superseded_by.is_none(),
        "proposal disposition links require an authoritative disposition record"
    );
    let mut lineage = BTreeSet::new();
    for assertion_id in &proposal.builds_on {
        anyhow::ensure!(
            lineage.insert(assertion_id.as_str()),
            "builds_on assertion {} is repeated",
            assertion_id.as_str()
        );
    }
    validate_optional_confidence_score(proposal.confidence)?;
    Ok(())
}

pub fn effective_proposal_state(
    proposal: &ProposalCard,
    assertions: &[AssertionRecord],
    dispositions: &[DispositionRecord],
) -> PromotionState {
    if proposal.promotion_state != PromotionState::Proposed {
        return proposal.promotion_state;
    }
    dispositions
        .iter()
        .find(|record| record.proposal_id == proposal.id)
        .map_or_else(
            || {
                if assertions
                    .iter()
                    .any(|record| record.proposal_id == proposal.id)
                {
                    PromotionState::Asserted
                } else {
                    proposal.promotion_state
                }
            },
            |record| match record.decision {
                super::DispositionDecision::Accepted => PromotionState::Accepted,
                super::DispositionDecision::Rejected => PromotionState::Rejected,
                super::DispositionDecision::Deferred => PromotionState::Deferred,
            },
        )
}

#[allow(clippy::too_many_lines)]
pub fn validate_ideation_aggregate(aggregate: IdeationAggregate<'_>) -> anyhow::Result<()> {
    for contribution in aggregate.contributions {
        anyhow::ensure!(
            contribution.schema_version.0 == 1,
            "contribution schema_version must be 1"
        );
    }
    for packet in aggregate.synthesis_packets {
        anyhow::ensure!(
            packet.schema_version.0 == 1,
            "synthesis schema_version must be 1"
        );
    }
    ensure_immutable_ids(
        "proposal",
        aggregate
            .proposals
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    validate_legacy_records(
        aggregate.proposals,
        aggregate.dispositions,
        aggregate.legacy_policy,
    )?;
    for proposal in aggregate.proposals {
        anyhow::ensure!(
            proposal.schema_version.0 == 1,
            "proposal schema_version must be 1"
        );
        if proposal.promotion_state == PromotionState::Proposed {
            validate_proposal_intrinsic(proposal)?;
        }
    }
    ensure_immutable_ids(
        "assertion",
        aggregate
            .assertions
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    ensure_immutable_ids(
        "disposition",
        aggregate
            .dispositions
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    for disposition in aggregate.dispositions {
        validate_disposition_intrinsic(disposition)?;
    }
    let proposals = aggregate
        .proposals
        .iter()
        .map(|proposal| (proposal.id.as_str(), proposal))
        .collect::<BTreeMap<_, _>>();
    for disposition in aggregate.dispositions {
        if proposals
            .get(disposition.proposal_id.as_str())
            .is_some_and(|proposal| proposal.promotion_state == PromotionState::Proposed)
        {
            anyhow::ensure!(
                aggregate
                    .disposition_actor_ids
                    .iter()
                    .any(|id| id == &disposition.actor.id),
                "disposition actor is not in the repository allowlist"
            );
        }
    }
    validate_lineage(aggregate.proposals, aggregate.assertions)?;
    let synthesis_packets = aggregate
        .synthesis_packets
        .iter()
        .map(|packet| (packet.id.as_str(), packet))
        .collect::<BTreeMap<_, _>>();
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
        let packet = synthesis_packets
            .get(assertion.synthesis_packet_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("assertion synthesis packet does not exist"))?;
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
            let owners = aggregate
                .contributions
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
            let (claim, owner) = owners[0];
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
                !matches!(
                    claim.evidence_type,
                    super::IdeationEvidenceType::Unsupported
                        | super::IdeationEvidenceType::Exploratory
                ),
                "assertion claim {} must use a positive evidence type",
                claim_id.as_str()
            );
            for evidence_id in &claim.evidence_reference_ids {
                let evidence = owner
                    .evidence_references
                    .iter()
                    .find(|evidence| evidence.reference_id == *evidence_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "assertion evidence {} does not exist",
                            evidence_id.as_str()
                        )
                    })?;
                anyhow::ensure!(
                    evidence.evidence_type == claim.evidence_type,
                    "assertion evidence type does not match claim {}",
                    claim_id.as_str()
                );
                anyhow::ensure!(
                    !matches!(
                        evidence.evidence_type,
                        super::IdeationEvidenceType::Unsupported
                            | super::IdeationEvidenceType::Exploratory
                    ),
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
    let mut disposed = BTreeSet::new();
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
        anyhow::ensure!(
            disposed.insert(disposition.proposal_id.as_str()),
            "proposal {} has multiple dispositions",
            disposition.proposal_id.as_str()
        );
        if proposal.promotion_state != PromotionState::Proposed {
            continue;
        }
        anyhow::ensure!(
            disposition.decision != super::DispositionDecision::Accepted
                || aggregate
                    .assertions
                    .iter()
                    .any(|assertion| assertion.proposal_id == proposal.id),
            "accepted proposal {} must be asserted before disposition",
            proposal.id.as_str()
        );
    }
    Ok(())
}

fn validate_legacy_records(
    proposals: &[ProposalCard],
    dispositions: &[DispositionRecord],
    policy: LegacyProposalPolicy,
) -> anyhow::Result<()> {
    let mut terminals = proposals
        .iter()
        .filter(|proposal| proposal.promotion_state != PromotionState::Proposed)
        .collect::<Vec<_>>();
    if terminals.is_empty() {
        return Ok(());
    }
    anyhow::ensure!(
        policy == LegacyProposalPolicy::ShippedV1,
        "terminal proposal rows are forbidden by the modern-only lifecycle policy"
    );
    terminals.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let mut hasher = Sha256::new();
    for proposal in terminals {
        hasher.update(serde_json::to_vec(proposal).expect("proposal serialization is infallible"));
        hasher.update(b"\n");
    }
    anyhow::ensure!(
        format!("{:x}", hasher.finalize()) == SHIPPED_LEGACY_TERMINAL_DIGEST_V1,
        "terminal proposal rows do not match the frozen shipped-v1 fingerprint"
    );
    let terminal_ids = proposals
        .iter()
        .filter(|proposal| proposal.promotion_state != PromotionState::Proposed)
        .map(|proposal| proposal.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut audit = dispositions
        .iter()
        .filter(|disposition| terminal_ids.contains(disposition.proposal_id.as_str()))
        .collect::<Vec<_>>();
    audit.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let mut hasher = Sha256::new();
    for disposition in audit {
        hasher.update(
            serde_json::to_vec(disposition).expect("disposition serialization is infallible"),
        );
        hasher.update(b"\n");
    }
    anyhow::ensure!(
        format!("{:x}", hasher.finalize()) == SHIPPED_LEGACY_DISPOSITION_DIGEST_V1,
        "disposition rows do not match the frozen shipped-v1 disposition audit"
    );
    Ok(())
}

fn validate_lineage(
    proposals: &[ProposalCard],
    assertion_records: &[AssertionRecord],
) -> anyhow::Result<()> {
    let assertions = assertion_records
        .iter()
        .map(|assertion| (assertion.id.as_str(), assertion.proposal_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let edges = proposals
        .iter()
        .map(|proposal| {
            let ancestors = proposal
                .builds_on
                .iter()
                .map(|id| {
                    assertions.get(id.as_str()).copied().ok_or_else(|| {
                        anyhow::anyhow!("builds_on assertion {} does not exist", id.as_str())
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            Ok((proposal.id.as_str(), ancestors))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for proposal in edges.keys() {
        visit(proposal, &edges, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn ensure_immutable_ids<'a, T: Serialize + 'a>(
    kind: &str,
    records: impl IntoIterator<Item = (&'a str, &'a T)>,
) -> anyhow::Result<()> {
    let mut seen = BTreeMap::new();
    for (id, record) in records {
        let value = serde_json::to_value(record)?;
        anyhow::ensure!(
            seen.insert(id, value).is_none(),
            "duplicate immutable {kind} id {id}"
        );
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
