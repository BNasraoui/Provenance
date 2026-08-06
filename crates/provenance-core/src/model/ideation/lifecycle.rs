use super::PromotionState;
use crate::model::{
    validate_optional_confidence_score, Contribution, DispositionRecord, ProposalCard,
    SchemaVersion, ScopeId, StableId, SynthesisPacket,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod aggregate_validation;
mod assertion_validation;
mod legacy_validation;
mod lineage_validation;

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

pub fn validate_ideation_aggregate(aggregate: IdeationAggregate<'_>) -> anyhow::Result<()> {
    aggregate_validation::validate(aggregate)
}
