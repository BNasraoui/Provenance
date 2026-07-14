use super::common::{ideation_target, parse_json_arg, stable_ids, warn_if_skills_missing};
use crate::cli::ideation::ProposalsCommand;
use crate::output;
use provenance_core::{
    AssertionId, IdeationEvidenceReference, PromotionState, ProposalTraceability, ProposalType,
    ScopeId, StableId,
};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateAssertionInput, CreateProposalCardInput, StateStore},
};

pub(super) fn handle(command: ProposalsCommand, quiet: bool) -> anyhow::Result<()> {
    match command {
        ProposalsCommand::Create {
            repo,
            scope,
            id,
            proposal_key,
            proposal_type,
            title,
            summary,
            confidence,
            target_type,
            target_id,
            source_id,
            evidence_json,
            supporting_claim_id,
            builds_on,
            format,
        } => {
            warn_if_skills_missing(&repo, quiet)?;
            let store = StateStore::new(ProvenanceLayout::new(repo));
            let input = CreateProposalCardInput {
                scope_id: ScopeId::new(scope)?,
                id: StableId::new(id)?,
                proposal_key,
                proposal_type: ProposalType::parse(&proposal_type)?,
                title,
                summary,
                confidence,
                traceability: ProposalTraceability {
                    target: ideation_target(&target_type, target_id)?,
                    source_ids: stable_ids(source_id)?,
                    evidence_references: parse_json_arg::<Vec<IdeationEvidenceReference>>(
                        "evidence-json",
                        &evidence_json,
                    )?,
                    supporting_claim_ids: stable_ids(supporting_claim_id)?,
                },
                builds_on: builds_on
                    .into_iter()
                    .map(AssertionId::new)
                    .collect::<anyhow::Result<_>>()?,
                duplicate_of: None,
                superseded_by: None,
            };
            let proposal = store.create_proposal_card(input)?;
            output::print(format, &proposal)?;
        }
        ProposalsCommand::Assert {
            repo,
            scope,
            id,
            proposal_id,
            synthesis_packet_id,
            supporting_claim_id,
            format,
        } => {
            warn_if_skills_missing(&repo, quiet)?;
            let assertion = StateStore::new(ProvenanceLayout::new(repo)).assert_proposal(
                CreateAssertionInput {
                    scope_id: ScopeId::new(scope)?,
                    id: AssertionId::new(id)?,
                    proposal_id: StableId::new(proposal_id)?,
                    synthesis_packet_id: StableId::new(synthesis_packet_id)?,
                    supporting_claim_ids: stable_ids(supporting_claim_id)?,
                },
            )?;
            output::print(format, &assertion)?;
        }
        ProposalsCommand::List {
            repo,
            scope,
            promotion_state,
            format,
        } => {
            warn_if_skills_missing(&repo, quiet)?;
            let mut proposals = StateStore::new(ProvenanceLayout::new(repo))
                .list_proposal_cards(&ScopeId::new(scope)?)?;
            if let Some(state) = promotion_state {
                let state = PromotionState::parse(&state)?;
                proposals.retain(|proposal| proposal.promotion_state == state);
            }
            output::print(format, &proposals)?;
        }
    }
    Ok(())
}
