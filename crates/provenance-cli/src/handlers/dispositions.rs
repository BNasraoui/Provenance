use super::common::canonical_artifact;
use crate::cli::ideation::DispositionsCommand;
use crate::output;
use provenance_core::{DispositionActor, DispositionDecision, IdentityType, ScopeId, StableId};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateDispositionInput, StateStore},
};

pub(super) fn handle(command: DispositionsCommand) -> anyhow::Result<()> {
    match command {
        DispositionsCommand::Create {
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
            let disposition = StateStore::new(ProvenanceLayout::new(repo)).create_disposition(
                CreateDispositionInput {
                    scope_id: ScopeId::new(scope)?,
                    id: StableId::new(id)?,
                    proposal_id: StableId::new(proposal_id)?,
                    decision: DispositionDecision::parse(&decision)?,
                    rationale,
                    actor: DispositionActor {
                        identity_type: IdentityType::parse(&actor_type)?,
                        id: actor_id,
                        name: actor_name,
                    },
                    canonical_artifact: canonical_artifact(
                        canonical_artifact_type,
                        canonical_artifact_id,
                    )?,
                },
            )?;
            output::print(format, &disposition)?;
        }
        DispositionsCommand::List {
            repo,
            scope,
            format,
        } => {
            let store = StateStore::new(ProvenanceLayout::new(repo));
            let scope_id = ScopeId::new(scope)?;
            let dispositions = store.with_repository_publication(|| {
                store.validate_ideation_scope(&scope_id)?;
                store.list_dispositions(&scope_id)
            })?;
            output::print(format, &dispositions)?;
        }
    }
    Ok(())
}
