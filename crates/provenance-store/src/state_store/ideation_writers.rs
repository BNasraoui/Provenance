use super::{
    CreateContributionInput, CreatePromotionDecisionInput, CreateProposalCardInput,
    CreateSynthesisPacketInput, StateStore,
};
use crate::{jsonl, shards};
use provenance_core::{
    Contribution, PromotionDecisionRecord, PromotionState, ProposalCard, SchemaVersion,
    SynthesisPacket,
};

impl StateStore {
    pub fn create_contribution(
        &self,
        input: CreateContributionInput,
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
        let mut records = self.list_contributions(&scope_id)?;
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
        anyhow::ensure!(
            !records.iter().any(|record| record.id == contribution.id),
            "contribution already exists"
        );
        records.push(contribution.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(
            &shards::contributions_path(&self.layout, &scope_id),
            &records,
        )?;
        Ok(contribution)
    }

    pub fn create_synthesis_packet(
        &self,
        input: CreateSynthesisPacketInput,
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
        let mut records = self.list_synthesis_packets(&scope_id)?;
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
        anyhow::ensure!(
            !records
                .iter()
                .any(|record| record.id == synthesis_packet.id),
            "synthesis packet already exists"
        );
        records.push(synthesis_packet.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(
            &shards::synthesis_packets_path(&self.layout, &scope_id),
            &records,
        )?;
        Ok(synthesis_packet)
    }

    pub fn create_proposal_card(
        &self,
        input: CreateProposalCardInput,
    ) -> anyhow::Result<ProposalCard> {
        let CreateProposalCardInput {
            scope_id,
            id,
            proposal_key,
            proposal_type,
            title,
            summary,
            traceability,
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
            | PromotionState::Accepted
            | PromotionState::Rejected
            | PromotionState::Deferred => {}
        }
        let mut records = self.list_proposal_cards(&scope_id)?;
        let proposal = ProposalCard {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            proposal_key,
            proposal_type,
            title,
            summary,
            traceability,
            promotion_state,
            duplicate_of,
            superseded_by,
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == proposal.id),
            "proposal already exists"
        );
        records.push(proposal.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(
            &shards::proposal_cards_path(&self.layout, &scope_id),
            &records,
        )?;
        Ok(proposal)
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
        let mut proposals = self.list_proposal_cards(&scope_id)?;
        let proposal = proposals
            .iter_mut()
            .find(|proposal| proposal.id == proposal_id)
            .ok_or_else(|| anyhow::anyhow!("proposal does not exist"))?;
        let mut records = self.list_promotion_decisions(&scope_id)?;
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
            !records
                .iter()
                .any(|record| record.id == promotion_decision.id),
            "promotion decision already exists"
        );
        proposal.promotion_state = match decision {
            provenance_core::PromotionDecision::Accepted => PromotionState::Accepted,
            provenance_core::PromotionDecision::Rejected => PromotionState::Rejected,
            provenance_core::PromotionDecision::Deferred => PromotionState::Deferred,
        };
        proposals.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(
            &shards::proposal_cards_path(&self.layout, &scope_id),
            &proposals,
        )?;
        records.push(promotion_decision.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(
            &shards::promotion_decisions_path(&self.layout, &scope_id),
            &records,
        )?;
        Ok(promotion_decision)
    }
}
