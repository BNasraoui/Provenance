use super::{
    serde_name, CreateAssertionInput, CreateContributionInput, CreateDispositionInput,
    CreateProposalCardInput, CreateSynthesisPacketInput, StateStore,
};
use crate::shards;
use provenance_core::{
    validate_optional_confidence_score, AssertionRecord, Contribution, DispositionRecord,
    PromotionState, ProposalCard, SchemaVersion, ScopeId, StableId, SynthesisPacket,
};

impl StateStore {
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
        let path = shards::contributions_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<Contribution>| {
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
            if let Some(index) = records
                .iter()
                .position(|record| record.id == contribution.id)
            {
                anyhow::ensure!(replace, "contribution already exists");
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
        let path = shards::synthesis_packets_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<SynthesisPacket>| {
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
            if let Some(index) = records
                .iter()
                .position(|record| record.id == synthesis_packet.id)
            {
                anyhow::ensure!(replace, "synthesis packet already exists");
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
        let CreateProposalCardInput {
            scope_id,
            id,
            proposal_key,
            proposal_type,
            title,
            summary,
            confidence,
            traceability,
            builds_on,
            promotion_state,
            duplicate_of,
            superseded_by,
        } = input;
        match promotion_state {
            PromotionState::Duplicate => {
                anyhow::ensure!(
                    duplicate_of.is_some(),
                    "duplicate proposals must set duplicate_of"
                );
            }
            PromotionState::Superseded => {
                anyhow::ensure!(
                    superseded_by.is_some(),
                    "superseded proposals must set superseded_by"
                );
            }
            PromotionState::Proposed
            | PromotionState::Asserted
            | PromotionState::Accepted
            | PromotionState::Rejected
            | PromotionState::Deferred => {}
        }
        let confidence = validate_optional_confidence_score(confidence)?;
        let candidate = ProposalCard {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id: id.clone(),
            proposal_key: proposal_key.clone(),
            proposal_type,
            title: title.clone(),
            summary: summary.clone(),
            confidence,
            traceability: traceability.clone(),
            promotion_state,
            builds_on: builds_on.clone(),
            duplicate_of: duplicate_of.clone(),
            superseded_by: superseded_by.clone(),
        };
        provenance_core::validate_proposal_intrinsic(&candidate)?;
        let mut proposals = self.list_proposal_definitions(&scope_id)?;
        proposals.push(candidate);
        provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
            legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
            disposition_actor_ids: &self.manifest()?.disposition_actor_ids,
            contributions: &self.list_contributions(&scope_id)?,
            synthesis_packets: &self.list_synthesis_packets(&scope_id)?,
            proposals: &proposals,
            assertions: &self.list_assertion_records(&scope_id)?,
            dispositions: &self.list_dispositions(&scope_id)?,
        })?;
        let path = shards::proposal_cards_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<ProposalCard>| {
            let proposal = ProposalCard {
                schema_version: SchemaVersion(1),
                scope_id: scope_id.clone(),
                id,
                proposal_key,
                proposal_type,
                title,
                summary,
                confidence,
                traceability,
                promotion_state,
                builds_on,
                duplicate_of,
                superseded_by,
            };
            if let Some(index) = records.iter().position(|record| record.id == proposal.id) {
                anyhow::bail!(
                    "proposal {} already exists and is immutable",
                    records[index].id.as_str()
                );
            }
            records.push(proposal.clone());
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(proposal)
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
            self.list_assertion_records(&scope_id)?
                .iter()
                .any(|assertion| assertion.proposal_id == proposal_id),
            "proposal must be asserted before disposition"
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
