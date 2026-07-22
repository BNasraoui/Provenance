use super::common::{ideation_target, parse_json_arg, stable_ids, warn_if_skills_missing};
use crate::cli::ideation::ProposalsCommand;
use crate::output;
use provenance_core::{
    IdeationEvidenceReference, PromotionState, ProposalTraceability, ProposalType, ScopeId,
    StableId,
};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateProposalCardInput, ProposalDemand, StateStore},
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
            promotion_state,
            duplicate_of,
            superseded_by,
            replace,
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
                promotion_state: PromotionState::parse(&promotion_state)?,
                duplicate_of: duplicate_of.map(StableId::new).transpose()?,
                superseded_by: superseded_by.map(StableId::new).transpose()?,
            };
            let proposal = if replace {
                store.upsert_proposal_card(input)?
            } else {
                store.create_proposal_card(input)?
            };
            output::print(format, &proposal)?;
        }
        ProposalsCommand::List {
            repo,
            scope,
            format,
        } => {
            warn_if_skills_missing(&repo, quiet)?;
            let proposals = StateStore::new(ProvenanceLayout::new(repo))
                .list_proposal_cards(&ScopeId::new(scope)?)?;
            output::print(format, &proposals)?;
        }
        ProposalsCommand::Surface {
            repo,
            scope,
            changed_path,
            target_type,
            target_id,
            format,
        } => {
            warn_if_skills_missing(&repo, quiet)?;
            let targets = match (target_type, target_id) {
                (Some(target_type), Some(target_id)) => {
                    vec![ideation_target(&target_type, target_id)?]
                }
                (None, None) => Vec::new(),
                _ => anyhow::bail!("--target-type and --target-id must be provided together"),
            };
            let surfaced = StateStore::new(ProvenanceLayout::new(repo)).surface_proposals(
                &ScopeId::new(scope)?,
                &ProposalDemand::new(changed_path, targets),
            )?;
            output::print(format, &surfaced)?;
        }
    }
    Ok(())
}
