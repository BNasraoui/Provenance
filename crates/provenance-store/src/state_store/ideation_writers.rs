use super::{
    read_jsonl, CreateAssertionInput, CreateContributionInput, CreatePromotionDecisionInput,
    CreateProposalCardInput, CreateSynthesisPacketInput, StateStore,
};
use crate::shards;
use provenance_core::{
    validate_ideation_aggregate, validate_optional_confidence_score, validate_proposal_intrinsic,
    AssertionRecord, Contribution, DispositionRecord, IdeationAggregate, PromotionState,
    ProposalCard, SchemaVersion, SynthesisPacket,
};

impl StateStore {
    pub fn create_contribution(
        &self,
        input: CreateContributionInput,
    ) -> anyhow::Result<Contribution> {
        let scope = input.scope_id.clone();
        self.with_ideation_lock(&scope, || self.write_contribution(input, false))
    }

    pub fn upsert_contribution(
        &self,
        input: CreateContributionInput,
    ) -> anyhow::Result<Contribution> {
        let scope = input.scope_id.clone();
        self.with_ideation_lock(&scope, || self.write_contribution(input, true))
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
        let direct: Vec<Contribution> = read_jsonl(&path)?;
        if self
            .list_contributions(&scope_id)?
            .iter()
            .any(|record| record.id == id)
            && !direct.iter().any(|record| record.id == id)
        {
            anyhow::bail!("landed contribution cannot be replaced through the direct writer");
        }
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
                let asserted_claims = self
                    .list_assertion_records(&scope_id)?
                    .into_iter()
                    .flat_map(|assertion| assertion.supporting_claim_ids)
                    .collect::<std::collections::BTreeSet<_>>();
                anyhow::ensure!(
                    !records[index]
                        .material_claims
                        .iter()
                        .any(|claim| asserted_claims.contains(&claim.claim_id)),
                    "contribution supplies durable assertion evidence and cannot be replaced"
                );
                records[index] = contribution.clone();
            } else {
                records.push(contribution.clone());
            }
            let mut all = self.list_contributions(&scope_id)?;
            if let Some(index) = all.iter().position(|record| record.id == contribution.id) {
                all[index] = contribution.clone();
            } else {
                all.push(contribution.clone());
            }
            validate_ideation_aggregate(IdeationAggregate {
                contributions: &all,
                synthesis_packets: &self.list_synthesis_packets(&scope_id)?,
                proposals: &self.list_proposal_records(&scope_id)?,
                assertions: &self.list_assertion_records(&scope_id)?,
                dispositions: &self.list_promotion_decisions(&scope_id)?,
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
        self.with_ideation_lock(&scope, || self.write_synthesis_packet(input, false))
    }

    pub fn upsert_synthesis_packet(
        &self,
        input: CreateSynthesisPacketInput,
    ) -> anyhow::Result<SynthesisPacket> {
        let scope = input.scope_id.clone();
        self.with_ideation_lock(&scope, || self.write_synthesis_packet(input, true))
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
        let direct: Vec<SynthesisPacket> = read_jsonl(&path)?;
        if self
            .list_synthesis_packets(&scope_id)?
            .iter()
            .any(|record| record.id == id)
            && !direct.iter().any(|record| record.id == id)
        {
            anyhow::bail!("landed synthesis packet cannot be replaced through the direct writer");
        }
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
                anyhow::ensure!(
                    !self
                        .list_assertion_records(&scope_id)?
                        .iter()
                        .any(|assertion| assertion.synthesis_packet_id == records[index].id),
                    "synthesis packet adjudicates a durable assertion and cannot be replaced"
                );
                records[index] = synthesis_packet.clone();
            } else {
                records.push(synthesis_packet.clone());
            }
            let mut all = self.list_synthesis_packets(&scope_id)?;
            if let Some(index) = all
                .iter()
                .position(|record| record.id == synthesis_packet.id)
            {
                all[index] = synthesis_packet.clone();
            } else {
                all.push(synthesis_packet.clone());
            }
            validate_ideation_aggregate(IdeationAggregate {
                contributions: &self.list_contributions(&scope_id)?,
                synthesis_packets: &all,
                proposals: &self.list_proposal_records(&scope_id)?,
                assertions: &self.list_assertion_records(&scope_id)?,
                dispositions: &self.list_promotion_decisions(&scope_id)?,
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
        self.with_ideation_lock(&scope, || self.write_proposal_card(input))
    }

    fn write_proposal_card(&self, input: CreateProposalCardInput) -> anyhow::Result<ProposalCard> {
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
            duplicate_of,
            superseded_by,
        } = input;
        anyhow::ensure!(
            duplicate_of.is_none() && superseded_by.is_none(),
            "proposal disposition links require an authoritative disposition record"
        );
        let confidence = validate_optional_confidence_score(confidence)?;
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
            promotion_state: PromotionState::Proposed,
            builds_on,
            duplicate_of,
            superseded_by,
        };
        validate_proposal_intrinsic(&proposal)?;
        anyhow::ensure!(
            !self
                .list_proposal_records(&scope_id)?
                .iter()
                .any(|record| record.id == proposal.id),
            "proposal already exists and is immutable"
        );
        let path = shards::proposal_cards_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<ProposalCard>| {
            records.push(proposal.clone());
            let mut all = self.list_proposal_records(&scope_id)?;
            all.push(proposal.clone());
            validate_ideation_aggregate(IdeationAggregate {
                contributions: &self.list_contributions(&scope_id)?,
                synthesis_packets: &self.list_synthesis_packets(&scope_id)?,
                proposals: &all,
                assertions: &self.list_assertion_records(&scope_id)?,
                dispositions: &self.list_promotion_decisions(&scope_id)?,
            })?;
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(proposal)
        })
    }

    pub fn assert_proposal(&self, input: CreateAssertionInput) -> anyhow::Result<AssertionRecord> {
        let scope = input.scope_id.clone();
        self.with_ideation_lock(&scope, || self.write_assertion(input))
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
                .list_promotion_decisions(&input.scope_id)?
                .iter()
                .any(|record| record.proposal_id == assertion.proposal_id),
            "a disposed proposal cannot be asserted"
        );
        let path = shards::assertion_records_path(&self.layout, &input.scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<AssertionRecord>| {
            anyhow::ensure!(
                !self
                    .list_assertion_records(&input.scope_id)?
                    .iter()
                    .any(|record| {
                        record.id == assertion.id || record.proposal_id == assertion.proposal_id
                    }),
                "assertion identity or proposal assertion already exists"
            );
            records.push(assertion.clone());
            let mut all = self.list_assertion_records(&input.scope_id)?;
            all.push(assertion.clone());
            validate_ideation_aggregate(IdeationAggregate {
                contributions: &self.list_contributions(&input.scope_id)?,
                synthesis_packets: &self.list_synthesis_packets(&input.scope_id)?,
                proposals: &self.list_proposal_records(&input.scope_id)?,
                assertions: &all,
                dispositions: &self.list_promotion_decisions(&input.scope_id)?,
            })?;
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(assertion)
        })
    }

    pub fn create_promotion_decision(
        &self,
        input: CreatePromotionDecisionInput,
    ) -> anyhow::Result<DispositionRecord> {
        let scope = input.scope_id.clone();
        self.with_ideation_lock(&scope, || self.write_disposition(input))
    }

    fn write_disposition(
        &self,
        input: CreatePromotionDecisionInput,
    ) -> anyhow::Result<DispositionRecord> {
        let CreatePromotionDecisionInput {
            scope_id,
            id,
            proposal_id,
            decision,
            rationale,
            actor,
            canonical_artifact,
        } = input;
        let promotion_decision = DispositionRecord {
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
            self.list_proposal_cards(&scope_id)?
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
        let proposal = self
            .list_proposal_records(&scope_id)?
            .into_iter()
            .find(|proposal| proposal.id == proposal_id)
            .ok_or_else(|| anyhow::anyhow!("proposal does not exist"))?;
        provenance_core::ensure_authoritative_actor(
            &proposal,
            promotion_decision.actor.identity_type,
        )?;
        anyhow::ensure!(
            !self
                .list_promotion_decisions(&scope_id)?
                .iter()
                .any(|record| {
                    record.id == promotion_decision.id
                        || record.proposal_id == promotion_decision.proposal_id
                }),
            "proposal already has an authoritative disposition"
        );
        let decisions_path = shards::promotion_decisions_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&decisions_path, |records: &mut Vec<DispositionRecord>| {
            anyhow::ensure!(
                !records
                    .iter()
                    .any(|record| record.id == promotion_decision.id),
                "promotion decision already exists"
            );
            anyhow::ensure!(
                !records
                    .iter()
                    .any(|record| record.proposal_id == promotion_decision.proposal_id),
                "proposal already has an authoritative disposition"
            );
            records.push(promotion_decision.clone());
            let mut all = self.list_promotion_decisions(&scope_id)?;
            all.push(promotion_decision.clone());
            validate_ideation_aggregate(IdeationAggregate {
                contributions: &self.list_contributions(&scope_id)?,
                synthesis_packets: &self.list_synthesis_packets(&scope_id)?,
                proposals: &self.list_proposal_records(&scope_id)?,
                assertions: &self.list_assertion_records(&scope_id)?,
                dispositions: &all,
            })?;
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(promotion_decision)
        })
    }
}
