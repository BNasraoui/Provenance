use super::{
    serde_name, CreateContributionInput, CreatePromotionDecisionInput, CreateProposalCardInput,
    CreateSynthesisPacketInput, StateStore,
};
use crate::shards;
use provenance_core::{
    validate_optional_confidence_score, Contribution, PromotionDecisionRecord, PromotionState,
    ProposalCard, SchemaVersion, ScopeId, StableId, SynthesisPacket,
};

impl StateStore {
    pub fn create_contribution(
        &self,
        input: CreateContributionInput,
    ) -> anyhow::Result<Contribution> {
        self.write_contribution(input, false)
    }

    pub fn upsert_contribution(
        &self,
        input: CreateContributionInput,
    ) -> anyhow::Result<Contribution> {
        self.write_contribution(input, true)
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
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(contribution)
        })
    }

    pub fn create_synthesis_packet(
        &self,
        input: CreateSynthesisPacketInput,
    ) -> anyhow::Result<SynthesisPacket> {
        self.write_synthesis_packet(input, false)
    }

    pub fn upsert_synthesis_packet(
        &self,
        input: CreateSynthesisPacketInput,
    ) -> anyhow::Result<SynthesisPacket> {
        self.write_synthesis_packet(input, true)
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
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(synthesis_packet)
        })
    }

    pub fn create_proposal_card(
        &self,
        input: CreateProposalCardInput,
    ) -> anyhow::Result<ProposalCard> {
        self.write_proposal_card(input, false)
    }

    pub fn upsert_proposal_card(
        &self,
        input: CreateProposalCardInput,
    ) -> anyhow::Result<ProposalCard> {
        self.write_proposal_card(input, true)
    }

    fn write_proposal_card(
        &self,
        input: CreateProposalCardInput,
        replace: bool,
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
            promotion_state,
            builds_on,
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
        anyhow::ensure!(
            !(promotion_state == PromotionState::Accepted
                && behavior_changing_proposal(proposal_type)),
            "accepting a behavior-changing proposal requires a human promotion decision"
        );
        let confidence = validate_optional_confidence_score(confidence)?;
        let path = shards::proposal_cards_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<ProposalCard>| {
            let mut seen_ancestors = std::collections::BTreeSet::new();
            for ancestor in &builds_on {
                anyhow::ensure!(ancestor != &id, "proposal cannot build on itself");
                anyhow::ensure!(
                    seen_ancestors.insert(ancestor.as_str()),
                    "builds_on proposal {} is repeated",
                    ancestor.as_str()
                );
                let base = records.iter().find(|record| &record.id == ancestor);
                anyhow::ensure!(
                    base.is_some(),
                    "builds_on proposal {} does not exist",
                    ancestor.as_str()
                );
                anyhow::ensure!(
                    base.is_some_and(|record| record.promotion_state == PromotionState::Asserted),
                    "builds_on proposal {} must be asserted",
                    ancestor.as_str()
                );
            }
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
                anyhow::ensure!(replace, "proposal already exists");
                self.ensure_existing_proposal_card_replaceable(&scope_id, &records[index])?;
                records[index] = proposal.clone();
            } else {
                records.push(proposal.clone());
            }
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(proposal)
        })
    }

    pub fn create_promotion_decision(
        &self,
        input: CreatePromotionDecisionInput,
    ) -> anyhow::Result<PromotionDecisionRecord> {
        let CreatePromotionDecisionInput {
            scope_id,
            id,
            proposal_id,
            decision,
            rationale,
            actor,
            canonical_artifact,
        } = input;
        let promotion_decision = PromotionDecisionRecord {
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
        let proposal = self
            .list_proposal_cards(&scope_id)?
            .into_iter()
            .find(|proposal| proposal.id == proposal_id)
            .expect("proposal existence checked");
        anyhow::ensure!(
            !(decision == provenance_core::PromotionDecision::Accepted
                && behavior_changing_proposal(proposal.proposal_type)
                && promotion_decision.actor.identity_type != provenance_core::IdentityType::Human),
            "behavior-changing acceptance requires a human actor"
        );
        let decisions_path = shards::promotion_decisions_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(
            &decisions_path,
            |records: &mut Vec<PromotionDecisionRecord>| {
                anyhow::ensure!(
                    !records
                        .iter()
                        .any(|record| record.id == promotion_decision.id),
                    "promotion decision already exists"
                );
                Ok(())
            },
        )?;
        let proposals_path = shards::proposal_cards_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&proposals_path, |proposals: &mut Vec<ProposalCard>| {
            let proposal = proposals
                .iter_mut()
                .find(|proposal| proposal.id == proposal_id)
                .ok_or_else(|| anyhow::anyhow!("proposal does not exist"))?;
            proposal.promotion_state = match decision {
                provenance_core::PromotionDecision::Accepted => PromotionState::Accepted,
                provenance_core::PromotionDecision::Rejected => PromotionState::Rejected,
                provenance_core::PromotionDecision::Deferred => PromotionState::Deferred,
            };
            proposals.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(())
        })?;
        self.mutate_jsonl_records(
            &decisions_path,
            |records: &mut Vec<PromotionDecisionRecord>| {
                anyhow::ensure!(
                    !records
                        .iter()
                        .any(|record| record.id == promotion_decision.id),
                    "promotion decision already exists"
                );
                records.push(promotion_decision.clone());
                records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
                Ok(promotion_decision)
            },
        )
    }

    pub fn ensure_proposal_card_replaceable(
        &self,
        scope_id: &ScopeId,
        proposal_id: &StableId,
    ) -> anyhow::Result<()> {
        if let Some(existing) = self
            .list_proposal_cards(scope_id)?
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
            .list_promotion_decisions(scope_id)?
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
                "proposal {} has a human disposition or durable disposition and cannot be replaced; promotion_state is {state}",
                existing.id.as_str()
            );
        }

        let decision_ids = decisions
            .iter()
            .map(|decision| decision.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "proposal {} has a human disposition or durable disposition and cannot be replaced; promotion_state is {state}; promotion decision(s): {decision_ids}",
            existing.id.as_str()
        );
    }
}

const fn behavior_changing_proposal(proposal_type: provenance_core::ProposalType) -> bool {
    matches!(
        proposal_type,
        provenance_core::ProposalType::RequirementCandidate
            | provenance_core::ProposalType::ResolutionCandidate
            | provenance_core::ProposalType::RuleCandidate
    )
}
