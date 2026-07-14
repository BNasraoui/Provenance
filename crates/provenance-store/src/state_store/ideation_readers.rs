use super::{read_jsonl, IdeationLandingBatch, StateStore};
use crate::shards;
use provenance_core::{
    AssertionRecord, Contribution, DispositionRecord, PromotionDecision, PromotionState,
    ProposalCard, ProposalView, ScopeId, SynthesisPacket,
};

impl StateStore {
    pub fn list_contributions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Contribution>> {
        let mut records = read_jsonl(&shards::contributions_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay(&mut records, batch.contributions, |record| {
                record.id.as_str().to_owned()
            });
        }
        Ok(records)
    }

    pub fn list_synthesis_packets(&self, scope: &ScopeId) -> anyhow::Result<Vec<SynthesisPacket>> {
        let mut records = read_jsonl(&shards::synthesis_packets_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay(&mut records, batch.synthesis_packets, |record| {
                record.id.as_str().to_owned()
            });
        }
        Ok(records)
    }

    pub fn list_proposal_cards(&self, scope: &ScopeId) -> anyhow::Result<Vec<ProposalView>> {
        let proposals = self.list_proposal_records(scope)?;
        let assertions = self.list_assertion_records(scope)?;
        let dispositions = self.list_promotion_decisions(scope)?;
        Ok(proposals
            .iter()
            .map(|proposal| {
                let state = dispositions
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
                            PromotionDecision::Accepted => PromotionState::Accepted,
                            PromotionDecision::Rejected => PromotionState::Rejected,
                            PromotionDecision::Deferred => PromotionState::Deferred,
                        },
                    );
                proposal.project(state)
            })
            .collect())
    }

    pub fn list_proposal_definitions(&self, scope: &ScopeId) -> anyhow::Result<Vec<ProposalCard>> {
        self.list_proposal_records(scope)
    }

    pub fn list_assertion_records(&self, scope: &ScopeId) -> anyhow::Result<Vec<AssertionRecord>> {
        let mut records = read_jsonl(&shards::assertion_records_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay(&mut records, batch.assertions, |record| {
                record.id.as_str().to_owned()
            });
        }
        Ok(records)
    }

    pub fn list_promotion_decisions(
        &self,
        scope: &ScopeId,
    ) -> anyhow::Result<Vec<DispositionRecord>> {
        let mut records = read_jsonl(&shards::promotion_decisions_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay(&mut records, batch.dispositions, |record| {
                record.id.as_str().to_owned()
            });
        }
        Ok(records)
    }

    pub fn validate_ideation_scope(&self, scope: &ScopeId) -> anyhow::Result<()> {
        provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
            contributions: &self.list_contributions(scope)?,
            synthesis_packets: &self.list_synthesis_packets(scope)?,
            proposals: &self.list_proposal_records(scope)?,
            assertions: &self.list_assertion_records(scope)?,
            dispositions: &self.list_promotion_decisions(scope)?,
        })
    }

    pub(super) fn list_proposal_records(
        &self,
        scope: &ScopeId,
    ) -> anyhow::Result<Vec<ProposalCard>> {
        let mut records = read_jsonl(&shards::proposal_cards_path(&self.layout, scope))?;
        for batch in self.list_ideation_landings(scope)? {
            overlay(&mut records, batch.proposals, |record| {
                record.id.as_str().to_owned()
            });
        }
        Ok(records)
    }

    fn list_ideation_landings(&self, scope: &ScopeId) -> anyhow::Result<Vec<IdeationLandingBatch>> {
        read_jsonl(&shards::ideation_landings_path(&self.layout, scope))
    }
}

fn overlay<T>(records: &mut Vec<T>, incoming: Vec<T>, id: impl Fn(&T) -> String) {
    for record in incoming {
        if let Some(index) = records
            .iter()
            .position(|existing| id(existing) == id(&record))
        {
            records[index] = record;
        } else {
            records.push(record);
        }
    }
    records.sort_by_key(id);
}
