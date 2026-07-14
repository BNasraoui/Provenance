use super::{read_jsonl, IdeationLandingBatch, StateStore};
use crate::shards;
use provenance_core::{
    AssertionRecord, Contribution, DispositionRecord, PromotionDecision, PromotionState,
    ProposalCard, ProposalView, ScopeId, SynthesisPacket,
};
use serde::Serialize;
use std::collections::BTreeMap;

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
        self.validate_raw_ideation_history(scope)?;
        let proposals = self.list_proposal_records(scope)?;
        let dispositions = self.list_promotion_decisions(scope)?;
        for disposition in &dispositions {
            if proposals.iter().any(|proposal| {
                proposal.id == disposition.proposal_id
                    && proposal.promotion_state == PromotionState::Proposed
                    && matches!(
                        proposal.proposal_type,
                        provenance_core::ProposalType::RequirementCandidate
                            | provenance_core::ProposalType::ResolutionCandidate
                            | provenance_core::ProposalType::RuleCandidate
                    )
            }) {
                self.ensure_human_authority(&disposition.actor.id)?;
            }
        }
        provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
            contributions: &self.list_contributions(scope)?,
            synthesis_packets: &self.list_synthesis_packets(scope)?,
            proposals: &proposals,
            assertions: &self.list_assertion_records(scope)?,
            dispositions: &dispositions,
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

    pub(super) fn has_landed_contribution(
        &self,
        scope: &ScopeId,
        id: &provenance_core::StableId,
    ) -> anyhow::Result<bool> {
        Ok(self
            .list_ideation_landings(scope)?
            .iter()
            .any(|batch| batch.contributions.iter().any(|record| record.id == *id)))
    }

    pub(super) fn has_landed_synthesis(
        &self,
        scope: &ScopeId,
        id: &provenance_core::StableId,
    ) -> anyhow::Result<bool> {
        Ok(self.list_ideation_landings(scope)?.iter().any(|batch| {
            batch
                .synthesis_packets
                .iter()
                .any(|record| record.id == *id)
        }))
    }

    pub(super) fn validate_raw_ideation_history(&self, scope: &ScopeId) -> anyhow::Result<()> {
        let direct_contributions: Vec<Contribution> =
            read_jsonl(&shards::contributions_path(&self.layout, scope))?;
        let direct_synthesis: Vec<SynthesisPacket> =
            read_jsonl(&shards::synthesis_packets_path(&self.layout, scope))?;
        let direct_proposals: Vec<ProposalCard> =
            read_jsonl(&shards::proposal_cards_path(&self.layout, scope))?;
        let direct_assertions: Vec<AssertionRecord> =
            read_jsonl(&shards::assertion_records_path(&self.layout, scope))?;
        let direct_dispositions: Vec<DispositionRecord> =
            read_jsonl(&shards::promotion_decisions_path(&self.layout, scope))?;
        let landings = self.list_ideation_landings(scope)?;

        validate_scope("contribution", scope, &direct_contributions, |r| {
            &r.scope_id
        })?;
        validate_scope("synthesis packet", scope, &direct_synthesis, |r| {
            &r.scope_id
        })?;
        validate_scope("proposal", scope, &direct_proposals, |r| &r.scope_id)?;
        validate_scope("assertion", scope, &direct_assertions, |r| &r.scope_id)?;
        validate_scope("disposition", scope, &direct_dispositions, |r| &r.scope_id)?;

        let mut proposals = BTreeMap::new();
        let mut assertions = BTreeMap::new();
        let mut dispositions = BTreeMap::new();
        for record in &direct_proposals {
            insert_immutable("proposal", record.id.as_str(), record, &mut proposals)?;
        }
        for record in &direct_assertions {
            insert_immutable("assertion", record.id.as_str(), record, &mut assertions)?;
        }
        for record in &direct_dispositions {
            insert_immutable("disposition", record.id.as_str(), record, &mut dispositions)?;
        }
        for batch in landings {
            validate_scope("contribution", scope, &batch.contributions, |r| &r.scope_id)?;
            validate_scope("synthesis packet", scope, &batch.synthesis_packets, |r| {
                &r.scope_id
            })?;
            validate_scope("proposal", scope, &batch.proposals, |r| &r.scope_id)?;
            validate_scope("assertion", scope, &batch.assertions, |r| &r.scope_id)?;
            validate_scope("disposition", scope, &batch.dispositions, |r| &r.scope_id)?;
            for proposal in &batch.proposals {
                provenance_core::validate_proposal_intrinsic(proposal)?;
            }
            for record in &batch.proposals {
                insert_immutable("proposal", record.id.as_str(), record, &mut proposals)?;
            }
            for record in &batch.assertions {
                insert_immutable("assertion", record.id.as_str(), record, &mut assertions)?;
            }
            for record in &batch.dispositions {
                insert_immutable("disposition", record.id.as_str(), record, &mut dispositions)?;
            }
        }
        Ok(())
    }
}

fn validate_scope<T>(
    kind: &str,
    expected: &ScopeId,
    records: &[T],
    scope: impl Fn(&T) -> &ScopeId,
) -> anyhow::Result<()> {
    for record in records {
        anyhow::ensure!(
            scope(record) == expected,
            "{kind} scope_id must match containing scope"
        );
    }
    Ok(())
}

fn insert_immutable<T: Serialize>(
    kind: &str,
    id: &str,
    record: &T,
    seen: &mut BTreeMap<String, serde_json::Value>,
) -> anyhow::Result<()> {
    let value = serde_json::to_value(record)?;
    if let Some(previous) = seen.insert(id.to_owned(), value.clone()) {
        anyhow::ensure!(
            previous == value,
            "divergent duplicate immutable {kind} id {id}"
        );
    }
    Ok(())
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
