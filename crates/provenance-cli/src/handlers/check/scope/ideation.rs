use crate::handlers::check::index::CheckIndex;
use crate::handlers::check::references::{check_ideation_target, check_scoped_reference};
use provenance_core::{
    Contribution, PromotionDecisionRecord, ProposalCard, ScopeId, SynthesisPacket,
};
use provenance_store::state_store::StateStore;

pub(super) struct Records {
    contributions: Vec<Contribution>,
    synthesis_packets: Vec<SynthesisPacket>,
    proposal_cards: Vec<ProposalCard>,
    promotion_decisions: Vec<PromotionDecisionRecord>,
}

impl Records {
    pub(super) fn load(store: &StateStore, scope_id: &ScopeId) -> anyhow::Result<Self> {
        Ok(Self {
            contributions: store.list_contributions(scope_id)?,
            synthesis_packets: store.list_synthesis_packets(scope_id)?,
            proposal_cards: store.list_proposal_cards(scope_id)?,
            promotion_decisions: store.list_promotion_decisions(scope_id)?,
        })
    }

    pub(super) fn add_to(&self, index: &mut CheckIndex) {
        for proposal in &self.proposal_cards {
            index.add_node(&proposal.scope_id, "proposal", &proposal.id);
        }
    }

    pub(super) fn validate(
        &self,
        index: &CheckIndex,
        scope_id: &ScopeId,
        dangling: &mut Vec<String>,
    ) {
        for contribution in &self.contributions {
            check_ideation_target(
                index,
                dangling,
                scope_id,
                &format!("contribution {}", contribution.id.as_str()),
                &contribution.target,
            );
        }
        for synthesis_packet in &self.synthesis_packets {
            check_ideation_target(
                index,
                dangling,
                scope_id,
                &format!("synthesis packet {}", synthesis_packet.id.as_str()),
                &synthesis_packet.target,
            );
        }
        for proposal in &self.proposal_cards {
            let owner = format!("proposal {}", proposal.id.as_str());
            check_ideation_target(
                index,
                dangling,
                scope_id,
                &owner,
                &proposal.traceability.target,
            );
            for source_id in &proposal.traceability.source_ids {
                check_scoped_reference(
                    index,
                    dangling,
                    scope_id,
                    &owner,
                    "source_id",
                    "source",
                    source_id,
                );
            }
            if let Some(duplicate_of) = &proposal.duplicate_of {
                check_scoped_reference(
                    index,
                    dangling,
                    scope_id,
                    &owner,
                    "duplicate_of",
                    "proposal",
                    duplicate_of,
                );
            }
            if let Some(superseded_by) = &proposal.superseded_by {
                check_scoped_reference(
                    index,
                    dangling,
                    scope_id,
                    &owner,
                    "superseded_by",
                    "proposal",
                    superseded_by,
                );
            }
        }
        for promotion_decision in &self.promotion_decisions {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &format!("promotion decision {}", promotion_decision.id.as_str()),
                "proposal",
                "proposal",
                &promotion_decision.proposal_id,
            );
        }
    }
}
