use super::{
    serde_name, CreateAssertionInput, CreateContributionInput, CreateDispositionInput,
    CreateProposalCardInput, CreateSynthesisPacketInput, StateStore,
};
use crate::shards;
use provenance_core::{
    validate_optional_confidence_score, AssertionRecord, Contribution, DispositionDecision,
    DispositionRecord, PromotionState, ProposalCard, SchemaVersion, ScopeId, StableId,
    SynthesisPacket,
};

impl StateStore {
    pub fn create_asserted_proposal(
        &self,
        proposal: CreateProposalCardInput,
        assertion: CreateAssertionInput,
    ) -> anyhow::Result<ProposalCard> {
        anyhow::ensure!(
            proposal.scope_id == assertion.scope_id,
            "proposal and assertion scope_id must match"
        );
        anyhow::ensure!(
            proposal.id == assertion.proposal_id,
            "assertion proposal_id must match proposal id"
        );
        let scope = proposal.scope_id.clone();
        self.with_lifecycle_lock(&scope, || {
            let proposal = proposal_from_input(proposal)?;
            let assertion = AssertionRecord {
                schema_version: SchemaVersion(1),
                scope_id: assertion.scope_id,
                id: assertion.id,
                proposal_id: assertion.proposal_id,
                synthesis_packet_id: assertion.synthesis_packet_id,
                supporting_claim_ids: assertion.supporting_claim_ids,
            };
            self.write_ideation_batch(
                &scope,
                super::IdeationLandingBatch {
                    contributions: Vec::new(),
                    synthesis_packets: Vec::new(),
                    proposals: vec![proposal.clone()],
                    assertions: vec![assertion],
                    dispositions: Vec::new(),
                },
                false,
            )?;
            Ok(proposal)
        })
    }

    pub fn assert_proposal(&self, input: CreateAssertionInput) -> anyhow::Result<AssertionRecord> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_assertion(input))
    }

    fn write_assertion(&self, input: CreateAssertionInput) -> anyhow::Result<AssertionRecord> {
        let assertion = AssertionRecord {
            schema_version: SchemaVersion(1),
            scope_id: input.scope_id.clone(),
            id: input.id,
            proposal_id: input.proposal_id,
            synthesis_packet_id: input.synthesis_packet_id,
            supporting_claim_ids: input.supporting_claim_ids,
        };
        anyhow::ensure!(
            !self
                .list_dispositions(&input.scope_id)?
                .iter()
                .any(|record| record.proposal_id == assertion.proposal_id),
            "a disposed proposal cannot re-enter assertion"
        );
        let path = shards::assertion_records_path(&self.layout, &input.scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<AssertionRecord>| {
            anyhow::ensure!(
                !records.iter().any(|record| {
                    record.id == assertion.id || record.proposal_id == assertion.proposal_id
                }),
                "assertion identity or proposal assertion already exists"
            );
            let mut assertions = self.list_assertion_records(&input.scope_id)?;
            assertions.push(assertion.clone());
            provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
                legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
                disposition_actor_ids: &self.manifest()?.disposition_actor_ids,
                contributions: &self.list_contributions(&input.scope_id)?,
                synthesis_packets: &self.list_synthesis_packets(&input.scope_id)?,
                proposals: &self.list_proposal_definitions(&input.scope_id)?,
                assertions: &assertions,
                dispositions: &self.list_dispositions(&input.scope_id)?,
            })?;
            records.push(assertion.clone());
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(assertion)
        })
    }

    pub fn create_contribution(
        &self,
        input: CreateContributionInput,
    ) -> anyhow::Result<Contribution> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_contribution(input, false))
    }

    pub fn upsert_contribution(
        &self,
        input: CreateContributionInput,
    ) -> anyhow::Result<Contribution> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_contribution(input, true))
    }

    fn write_contribution(
        &self,
        input: CreateContributionInput,
        replace: bool,
    ) -> anyhow::Result<Contribution> {
        let CreateContributionInput {
            scope_id,
            id,
            target,
            participant_slot,
            stance,
            strongest_finding,
            evidence_references,
            material_claims,
            risks,
            objections,
            challenges,
            suggested_artifact_changes,
            unsupported_recommendations,
            uncertainty,
            open_questions,
        } = input;
        for claim in &material_claims {
            validate_optional_confidence_score(claim.confidence)?;
        }
        let contribution = Contribution {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            target,
            participant_slot,
            stance,
            strongest_finding,
            evidence_references,
            material_claims,
            risks,
            objections,
            challenges,
            suggested_artifact_changes,
            unsupported_recommendations,
            uncertainty,
            open_questions,
        };
        let landed = self.list_ideation_landings(&scope_id)?.iter().any(|batch| {
            batch
                .contributions
                .iter()
                .any(|record| record.id == contribution.id)
        });
        if landed {
            anyhow::ensure!(replace, "contribution already exists");
            self.write_ideation_batch(
                &scope_id,
                super::IdeationLandingBatch {
                    contributions: vec![contribution.clone()],
                    synthesis_packets: Vec::new(),
                    proposals: Vec::new(),
                    assertions: Vec::new(),
                    dispositions: Vec::new(),
                },
                true,
            )?;
            return Ok(contribution);
        }
        let path = shards::contributions_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<Contribution>| {
            if let Some(index) = records
                .iter()
                .position(|record| record.id == contribution.id)
            {
                anyhow::ensure!(replace, "contribution already exists");
                super::ideation_batches::ensure_asserted_contribution_unchanged(
                    &records[index],
                    &contribution,
                    &self.list_assertion_records(&scope_id)?,
                )?;
                records[index] = contribution.clone();
            } else {
                records.push(contribution.clone());
            }
            let mut contributions = self.list_contributions(&scope_id)?;
            if let Some(index) = contributions
                .iter()
                .position(|record| record.id == contribution.id)
            {
                contributions[index] = contribution.clone();
            } else {
                contributions.push(contribution.clone());
            }
            provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
                legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
                disposition_actor_ids: &self.manifest()?.disposition_actor_ids,
                contributions: &contributions,
                synthesis_packets: &self.list_synthesis_packets(&scope_id)?,
                proposals: &self.list_proposal_definitions(&scope_id)?,
                assertions: &self.list_assertion_records(&scope_id)?,
                dispositions: &self.list_dispositions(&scope_id)?,
            })?;
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(contribution)
        })
    }

    pub fn create_synthesis_packet(
        &self,
        input: CreateSynthesisPacketInput,
    ) -> anyhow::Result<SynthesisPacket> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_synthesis_packet(input, false))
    }

    pub fn upsert_synthesis_packet(
        &self,
        input: CreateSynthesisPacketInput,
    ) -> anyhow::Result<SynthesisPacket> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_synthesis_packet(input, true))
    }

    fn write_synthesis_packet(
        &self,
        input: CreateSynthesisPacketInput,
        replace: bool,
    ) -> anyhow::Result<SynthesisPacket> {
        let CreateSynthesisPacketInput {
            scope_id,
            id,
            target,
            summary,
            consensus,
            contested_claims,
            minority_objections,
            evidence_gaps,
            unsupported_speculation,
            open_questions,
            suggested_artifacts,
            required_human_decisions,
        } = input;
        let synthesis_packet = SynthesisPacket {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            target,
            summary,
            consensus,
            contested_claims,
            minority_objections,
            evidence_gaps,
            unsupported_speculation,
            open_questions,
            suggested_artifacts,
            required_human_decisions,
        };
        let landed = self.list_ideation_landings(&scope_id)?.iter().any(|batch| {
            batch
                .synthesis_packets
                .iter()
                .any(|record| record.id == synthesis_packet.id)
        });
        if landed {
            anyhow::ensure!(replace, "synthesis packet already exists");
            self.write_ideation_batch(
                &scope_id,
                super::IdeationLandingBatch {
                    contributions: Vec::new(),
                    synthesis_packets: vec![synthesis_packet.clone()],
                    proposals: Vec::new(),
                    assertions: Vec::new(),
                    dispositions: Vec::new(),
                },
                true,
            )?;
            return Ok(synthesis_packet);
        }
        let path = shards::synthesis_packets_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<SynthesisPacket>| {
            if let Some(index) = records
                .iter()
                .position(|record| record.id == synthesis_packet.id)
            {
                anyhow::ensure!(replace, "synthesis packet already exists");
                super::ideation_batches::ensure_asserted_synthesis_unchanged(
                    &records[index],
                    &synthesis_packet,
                    &self.list_assertion_records(&scope_id)?,
                )?;
                records[index] = synthesis_packet.clone();
            } else {
                records.push(synthesis_packet.clone());
            }
            let mut synthesis_packets = self.list_synthesis_packets(&scope_id)?;
            if let Some(index) = synthesis_packets
                .iter()
                .position(|record| record.id == synthesis_packet.id)
            {
                synthesis_packets[index] = synthesis_packet.clone();
            } else {
                synthesis_packets.push(synthesis_packet.clone());
            }
            provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
                legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
                disposition_actor_ids: &self.manifest()?.disposition_actor_ids,
                contributions: &self.list_contributions(&scope_id)?,
                synthesis_packets: &synthesis_packets,
                proposals: &self.list_proposal_definitions(&scope_id)?,
                assertions: &self.list_assertion_records(&scope_id)?,
                dispositions: &self.list_dispositions(&scope_id)?,
            })?;
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(synthesis_packet)
        })
    }

    pub fn create_proposal_card(
        &self,
        input: CreateProposalCardInput,
    ) -> anyhow::Result<ProposalCard> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_proposal_card(input, false))
    }

    pub fn upsert_proposal_card(
        &self,
        input: CreateProposalCardInput,
    ) -> anyhow::Result<ProposalCard> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_proposal_card(input, true))
    }

    fn write_proposal_card(
        &self,
        input: CreateProposalCardInput,
        _replace: bool,
    ) -> anyhow::Result<ProposalCard> {
        let candidate = proposal_from_input(input)?;
        let mut proposals = self.list_proposal_definitions(&candidate.scope_id)?;
        proposals.push(candidate.clone());
        provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
            legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
            disposition_actor_ids: &self.manifest()?.disposition_actor_ids,
            contributions: &self.list_contributions(&candidate.scope_id)?,
            synthesis_packets: &self.list_synthesis_packets(&candidate.scope_id)?,
            proposals: &proposals,
            assertions: &self.list_assertion_records(&candidate.scope_id)?,
            dispositions: &self.list_dispositions(&candidate.scope_id)?,
        })?;
        let path = shards::proposal_cards_path(&self.layout, &candidate.scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<ProposalCard>| {
            if let Some(index) = records.iter().position(|record| record.id == candidate.id) {
                anyhow::bail!(
                    "proposal {} already exists and is immutable",
                    records[index].id.as_str()
                );
            }
            records.push(candidate.clone());
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(candidate)
        })
    }

    pub fn create_disposition(
        &self,
        input: CreateDispositionInput,
    ) -> anyhow::Result<DispositionRecord> {
        let scope = input.scope_id.clone();
        self.with_lifecycle_lock(&scope, || self.write_disposition(input))
    }

    fn write_disposition(
        &self,
        input: CreateDispositionInput,
    ) -> anyhow::Result<DispositionRecord> {
        let CreateDispositionInput {
            scope_id,
            id,
            proposal_id,
            decision,
            rationale,
            actor,
            canonical_artifact,
        } = input;
        let disposition = DispositionRecord {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            proposal_id: proposal_id.clone(),
            decision,
            rationale,
            actor,
            canonical_artifact,
        };
        anyhow::ensure!(
            self.list_proposal_definitions(&scope_id)?
                .iter()
                .any(|proposal| proposal.id == proposal_id),
            "proposal does not exist"
        );
        anyhow::ensure!(
            disposition.decision != DispositionDecision::Accepted
                || self
                    .list_assertion_records(&scope_id)?
                    .iter()
                    .any(|assertion| assertion.proposal_id == proposal_id),
            "accepted proposal must be asserted before disposition"
        );
        let mut dispositions = self.list_dispositions(&scope_id)?;
        anyhow::ensure!(
            !dispositions.iter().any(|record| {
                record.id == disposition.id || record.proposal_id == disposition.proposal_id
            }),
            "proposal already has an authoritative disposition"
        );
        dispositions.push(disposition.clone());
        provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
            legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
            disposition_actor_ids: &self.manifest()?.disposition_actor_ids,
            contributions: &self.list_contributions(&scope_id)?,
            synthesis_packets: &self.list_synthesis_packets(&scope_id)?,
            proposals: &self.list_proposal_definitions(&scope_id)?,
            assertions: &self.list_assertion_records(&scope_id)?,
            dispositions: &dispositions,
        })?;
        let dispositions_path = shards::dispositions_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(
            &dispositions_path,
            |records: &mut Vec<DispositionRecord>| {
                anyhow::ensure!(
                    !records.iter().any(|record| record.id == disposition.id),
                    "disposition already exists"
                );
                records.push(disposition.clone());
                records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
                Ok(disposition)
            },
        )
    }

    pub fn ensure_proposal_card_replaceable(
        &self,
        scope_id: &ScopeId,
        proposal_id: &StableId,
    ) -> anyhow::Result<()> {
        if let Some(existing) = self
            .list_proposal_definitions(scope_id)?
            .into_iter()
            .find(|proposal| proposal.id.as_str() == proposal_id.as_str())
        {
            self.ensure_existing_proposal_card_replaceable(scope_id, &existing)?;
        }
        Ok(())
    }

    fn ensure_existing_proposal_card_replaceable(
        &self,
        scope_id: &ScopeId,
        existing: &ProposalCard,
    ) -> anyhow::Result<()> {
        let decisions = self
            .list_dispositions(scope_id)?
            .into_iter()
            .filter(|decision| decision.proposal_id.as_str() == existing.id.as_str())
            .collect::<Vec<_>>();
        if existing.promotion_state == PromotionState::Proposed && decisions.is_empty() {
            return Ok(());
        }

        let state = serde_name(&existing.promotion_state)
            .unwrap_or_else(|_| format!("{:?}", existing.promotion_state));
        if decisions.is_empty() {
            anyhow::bail!(
                "proposal {} has a human disposition and cannot be replaced; promotion_state is {state}",
                existing.id.as_str()
            );
        }

        let decision_ids = decisions
            .iter()
            .map(|decision| decision.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "proposal {} has a human disposition and cannot be replaced; promotion_state is {state}; disposition(s): {decision_ids}",
            existing.id.as_str()
        );
    }
}

fn proposal_from_input(input: CreateProposalCardInput) -> anyhow::Result<ProposalCard> {
    match input.promotion_state {
        PromotionState::Duplicate => {
            anyhow::ensure!(
                input.duplicate_of.is_some(),
                "duplicate proposals must set duplicate_of"
            );
        }
        PromotionState::Superseded => {
            anyhow::ensure!(
                input.superseded_by.is_some(),
                "superseded proposals must set superseded_by"
            );
        }
        PromotionState::Proposed
        | PromotionState::Asserted
        | PromotionState::Accepted
        | PromotionState::Rejected
        | PromotionState::Deferred => {}
    }
    let proposal = ProposalCard {
        schema_version: SchemaVersion(1),
        scope_id: input.scope_id,
        id: input.id,
        proposal_key: input.proposal_key,
        proposal_type: input.proposal_type,
        title: input.title,
        summary: input.summary,
        confidence: validate_optional_confidence_score(input.confidence)?,
        traceability: input.traceability,
        promotion_state: input.promotion_state,
        builds_on: input.builds_on,
        duplicate_of: input.duplicate_of,
        superseded_by: input.superseded_by,
    };
    provenance_core::validate_proposal_intrinsic(&proposal)?;
    Ok(proposal)
}
