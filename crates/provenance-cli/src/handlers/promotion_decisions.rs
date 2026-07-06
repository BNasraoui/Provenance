use super::common::canonical_artifact;
use crate::cli::PromotionDecisionsCommand;
use crate::output;
use provenance_core::{IdentityType, PromotionActor, PromotionDecision, ScopeId, StableId};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreatePromotionDecisionInput, StateStore},
};

pub(super) fn handle(command: PromotionDecisionsCommand) -> anyhow::Result<()> {
    match command {
        PromotionDecisionsCommand::Create {
            repo,
            scope,
            id,
            proposal_id,
            decision,
            rationale,
            actor_id,
            actor_type,
            actor_name,
            canonical_artifact_type,
            canonical_artifact_id,
            format,
        } => {
            let promotion_decision = StateStore::new(ProvenanceLayout::new(repo))
                .create_promotion_decision(CreatePromotionDecisionInput {
                    scope_id: ScopeId::new(scope)?,
                    id: StableId::new(id)?,
                    proposal_id: StableId::new(proposal_id)?,
                    decision: PromotionDecision::parse(&decision)?,
                    rationale,
                    actor: PromotionActor {
                        identity_type: IdentityType::parse(&actor_type)?,
                        id: actor_id,
                        name: actor_name,
                    },
                    canonical_artifact: canonical_artifact(
                        canonical_artifact_type,
                        canonical_artifact_id,
                    )?,
                })?;
            output::print(format, &promotion_decision)?;
        }
        PromotionDecisionsCommand::List {
            repo,
            scope,
            format,
        } => {
            let decisions = StateStore::new(ProvenanceLayout::new(repo))
                .list_promotion_decisions(&ScopeId::new(scope)?)?;
            output::print(format, &decisions)?;
        }
    }
    Ok(())
}
