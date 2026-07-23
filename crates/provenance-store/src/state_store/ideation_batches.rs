use super::{read_jsonl, read_legacy_dispositions, IdeationLandingBatch, StateStore};
use crate::shards;
use provenance_core::{
    AssertionRecord, DispositionRecord, IdeationAggregate, ProposalCard, ScopeId,
};
use std::collections::{BTreeMap, BTreeSet};

impl StateStore {
    pub(super) fn with_lifecycle_lock<R>(
        &self,
        scope: &ScopeId,
        operation: impl FnOnce() -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        self.with_repository_publication(|| {
            crate::jsonl::with_advisory_lock(&self.layout.lifecycle_lock_path(scope), operation)
        })
    }

    pub(super) fn list_ideation_landings(
        &self,
        scope: &ScopeId,
    ) -> anyhow::Result<Vec<IdeationLandingBatch>> {
        read_jsonl(&shards::ideation_landings_path(&self.layout, scope))
    }

    pub fn land_ideation_batch(
        &self,
        scope: &ScopeId,
        incoming: IdeationLandingBatch,
        replace: bool,
    ) -> anyhow::Result<()> {
        self.with_lifecycle_lock(scope, || {
            self.write_ideation_batch(scope, incoming, replace)
        })
    }

    pub(super) fn write_ideation_batch(
        &self,
        scope: &ScopeId,
        incoming: IdeationLandingBatch,
        replace: bool,
    ) -> anyhow::Result<()> {
        ensure_scope(scope, &incoming)?;
        let path = shards::ideation_landings_path(&self.layout, scope);
        self.mutate_jsonl_records(&path, |landings: &mut Vec<IdeationLandingBatch>| {
            let mut contributions = self.list_contributions(scope)?;
            let mut synthesis_packets = self.list_synthesis_packets(scope)?;
            let mut proposals = self.list_proposal_definitions(scope)?;
            let mut assertions = self.list_assertion_records(scope)?;
            let mut dispositions = self.list_dispositions(scope)?;
            let existing_proposal_ids = proposals
                .iter()
                .map(|record| record.id.as_str().to_owned())
                .collect::<BTreeSet<_>>();
            let existing_assertion_ids = assertions
                .iter()
                .map(|record| record.id.as_str().to_owned())
                .collect::<BTreeSet<_>>();
            let existing_disposition_ids = dispositions
                .iter()
                .map(|record| record.id.as_str().to_owned())
                .collect::<BTreeSet<_>>();
            merge_immutable("proposal", &mut proposals, &incoming.proposals, |r| {
                r.id.as_str()
            })?;
            for replacement in &incoming.contributions {
                if let Some(existing) = contributions
                    .iter()
                    .find(|record| record.id == replacement.id)
                {
                    ensure_asserted_contribution_unchanged(existing, replacement, &assertions)?;
                }
            }
            for replacement in &incoming.synthesis_packets {
                if let Some(existing) = synthesis_packets
                    .iter()
                    .find(|record| record.id == replacement.id)
                {
                    ensure_asserted_synthesis_unchanged(existing, replacement, &assertions)?;
                }
            }
            merge_replaceable(
                "contribution",
                &mut contributions,
                &incoming.contributions,
                replace,
                |r| r.id.as_str(),
            )?;
            merge_replaceable(
                "synthesis packet",
                &mut synthesis_packets,
                &incoming.synthesis_packets,
                replace,
                |r| r.id.as_str(),
            )?;
            merge_immutable("assertion", &mut assertions, &incoming.assertions, |r| {
                r.id.as_str()
            })?;
            merge_immutable(
                "disposition",
                &mut dispositions,
                &incoming.dispositions,
                |r| r.id.as_str(),
            )?;
            ensure_qualifying_assertions(&proposals, &synthesis_packets, &assertions)?;
            for proposal in &incoming.proposals {
                provenance_core::validate_proposal_intrinsic(proposal)?;
            }
            let manifest = self.manifest()?;
            provenance_core::validate_ideation_aggregate(IdeationAggregate {
                legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
                disposition_actor_ids: &manifest.disposition_actor_ids,
                contributions: &contributions,
                synthesis_packets: &synthesis_packets,
                proposals: &proposals,
                assertions: &assertions,
                dispositions: &dispositions,
            })?;
            let mut persisted = incoming;
            persisted
                .proposals
                .retain(|record| !existing_proposal_ids.contains(record.id.as_str()));
            persisted
                .assertions
                .retain(|record| !existing_assertion_ids.contains(record.id.as_str()));
            persisted
                .dispositions
                .retain(|record| !existing_disposition_ids.contains(record.id.as_str()));
            landings.push(persisted);
            Ok(())
        })
    }

    pub fn validate_ideation_scope(&self, scope: &ScopeId) -> anyhow::Result<()> {
        let direct_proposals: Vec<ProposalCard> =
            read_jsonl(&shards::proposal_cards_path(&self.layout, scope))?;
        let direct_assertions: Vec<AssertionRecord> =
            read_jsonl(&shards::assertion_records_path(&self.layout, scope))?;
        let mut direct_dispositions: Vec<DispositionRecord> =
            read_jsonl(&shards::dispositions_path(&self.layout, scope))?;
        direct_dispositions.extend(read_legacy_dispositions(
            &shards::legacy_promotion_decisions_path(&self.layout, scope),
        )?);
        let mut proposals = BTreeMap::new();
        let mut assertions = BTreeMap::new();
        let mut dispositions = BTreeMap::new();
        insert_all(
            "proposal",
            &direct_proposals,
            |r| r.id.as_str(),
            &mut proposals,
        )?;
        insert_all(
            "assertion",
            &direct_assertions,
            |r| r.id.as_str(),
            &mut assertions,
        )?;
        insert_all(
            "disposition",
            &direct_dispositions,
            |r| r.id.as_str(),
            &mut dispositions,
        )?;
        for batch in self.list_ideation_landings(scope)? {
            insert_all(
                "proposal",
                &batch.proposals,
                |r| r.id.as_str(),
                &mut proposals,
            )?;
            insert_all(
                "assertion",
                &batch.assertions,
                |r| r.id.as_str(),
                &mut assertions,
            )?;
            insert_all(
                "disposition",
                &batch.dispositions,
                |r| r.id.as_str(),
                &mut dispositions,
            )?;
        }
        let proposals = self.list_proposal_definitions(scope)?;
        let assertions = self.list_assertion_records(scope)?;
        let dispositions = self.list_dispositions(scope)?;
        let has_modern_disposition = dispositions.iter().any(|disposition| {
            proposals.iter().any(|proposal| {
                proposal.id == disposition.proposal_id
                    && proposal.promotion_state == provenance_core::PromotionState::Proposed
            })
        });
        let disposition_actor_ids = if has_modern_disposition {
            self.manifest()?.disposition_actor_ids
        } else {
            Vec::new()
        };
        provenance_core::validate_ideation_aggregate(IdeationAggregate {
            legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
            disposition_actor_ids: &disposition_actor_ids,
            contributions: &self.list_contributions(scope)?,
            synthesis_packets: &self.list_synthesis_packets(scope)?,
            proposals: &proposals,
            assertions: &assertions,
            dispositions: &dispositions,
        })?;
        Ok(())
    }
}

pub(super) fn ensure_asserted_contribution_unchanged(
    existing: &provenance_core::Contribution,
    replacement: &provenance_core::Contribution,
    assertions: &[AssertionRecord],
) -> anyhow::Result<()> {
    if serde_json::to_value(existing)? == serde_json::to_value(replacement)? {
        return Ok(());
    }
    let referenced = assertions.iter().any(|assertion| {
        existing
            .material_claims
            .iter()
            .any(|claim| assertion.supporting_claim_ids.contains(&claim.claim_id))
    });
    anyhow::ensure!(
        !referenced,
        "contribution {} is referenced by an assertion and cannot be replaced",
        existing.id.as_str()
    );
    Ok(())
}

pub(super) fn ensure_asserted_synthesis_unchanged(
    existing: &provenance_core::SynthesisPacket,
    replacement: &provenance_core::SynthesisPacket,
    assertions: &[AssertionRecord],
) -> anyhow::Result<()> {
    if serde_json::to_value(existing)? == serde_json::to_value(replacement)? {
        return Ok(());
    }
    anyhow::ensure!(
        !assertions
            .iter()
            .any(|assertion| assertion.synthesis_packet_id == existing.id),
        "synthesis packet {} is referenced by an assertion and cannot be replaced",
        existing.id.as_str()
    );
    Ok(())
}

fn ensure_qualifying_assertions(
    proposals: &[ProposalCard],
    synthesis_packets: &[provenance_core::SynthesisPacket],
    assertions: &[AssertionRecord],
) -> anyhow::Result<()> {
    for proposal in proposals {
        let qualifying = synthesis_packets.iter().any(|packet| {
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
        });
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

fn insert_all<'a, T: serde::Serialize>(
    kind: &str,
    records: &'a [T],
    id: impl Fn(&'a T) -> &'a str,
    seen: &mut BTreeMap<String, serde_json::Value>,
) -> anyhow::Result<()> {
    for record in records {
        let record_id = id(record);
        let value = serde_json::to_value(record)?;
        anyhow::ensure!(
            seen.insert(record_id.to_owned(), value).is_none(),
            "duplicate immutable {kind} id {record_id}"
        );
    }
    Ok(())
}

fn ensure_scope(scope: &ScopeId, batch: &IdeationLandingBatch) -> anyhow::Result<()> {
    for (kind, actual) in batch
        .contributions
        .iter()
        .map(|r| ("contribution", &r.scope_id))
        .chain(
            batch
                .synthesis_packets
                .iter()
                .map(|r| ("synthesis packet", &r.scope_id)),
        )
        .chain(batch.proposals.iter().map(|r| ("proposal", &r.scope_id)))
        .chain(batch.assertions.iter().map(|r| ("assertion", &r.scope_id)))
        .chain(
            batch
                .dispositions
                .iter()
                .map(|r| ("disposition", &r.scope_id)),
        )
    {
        anyhow::ensure!(actual == scope, "{kind} scope_id must match landing scope");
    }
    Ok(())
}

fn merge_replaceable<T: Clone>(
    kind: &str,
    existing: &mut Vec<T>,
    incoming: &[T],
    replace: bool,
    id: impl Fn(&T) -> &str,
) -> anyhow::Result<()> {
    let mut incoming_ids = BTreeSet::new();
    for record in incoming {
        let record_id = id(record);
        anyhow::ensure!(
            incoming_ids.insert(record_id),
            "duplicate {kind} id {record_id} in batch"
        );
        if let Some(index) = existing.iter().position(|current| id(current) == record_id) {
            anyhow::ensure!(replace, "{kind} {record_id} already exists");
            existing[index] = record.clone();
        } else {
            existing.push(record.clone());
        }
    }
    Ok(())
}

fn merge_immutable<T: Clone + serde::Serialize>(
    kind: &str,
    existing: &mut Vec<T>,
    incoming: &[T],
    id: impl Fn(&T) -> &str,
) -> anyhow::Result<()> {
    let mut incoming_ids = BTreeSet::new();
    for record in incoming {
        let record_id = id(record);
        anyhow::ensure!(
            incoming_ids.insert(record_id),
            "duplicate {kind} id {record_id} in batch"
        );
        if let Some(current) = existing.iter().find(|current| id(current) == record_id) {
            anyhow::ensure!(
                serde_json::to_value(current)? == serde_json::to_value(record)?,
                "{kind} {record_id} already exists and is immutable"
            );
        } else {
            existing.push(record.clone());
        }
    }
    Ok(())
}

pub(super) fn overlay_records<T>(records: &mut Vec<T>, incoming: Vec<T>, id: impl Fn(&T) -> &str) {
    for record in incoming {
        if let Some(index) = records
            .iter()
            .position(|current| id(current) == id(&record))
        {
            records[index] = record;
        } else {
            records.push(record);
        }
    }
    records.sort_by(|a, b| id(a).cmp(id(b)));
}
