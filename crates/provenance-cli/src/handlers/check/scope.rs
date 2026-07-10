use super::index::CheckIndex;
use provenance_core::{Scope, ScopeId};
use provenance_store::state_store::StateStore;

mod collaboration;
mod core;
mod ideation;

struct ScopeRecords {
    scope_id: ScopeId,
    core: core::Records,
    collaboration: collaboration::Records,
    ideation: ideation::Records,
}

impl ScopeRecords {
    fn load(store: &StateStore, scope_id: &ScopeId) -> anyhow::Result<Self> {
        Ok(Self {
            scope_id: scope_id.clone(),
            core: core::Records::load(store, scope_id)?,
            collaboration: collaboration::Records::load(store, scope_id)?,
            ideation: ideation::Records::load(store, scope_id)?,
        })
    }
}

pub(super) fn validate(
    store: &StateStore,
    scopes: &[Scope],
    index: &mut CheckIndex,
    dangling: &mut Vec<String>,
) -> anyhow::Result<()> {
    let records = scopes
        .iter()
        .map(|scope| ScopeRecords::load(store, &scope.id))
        .collect::<anyhow::Result<Vec<_>>>()?;

    for scope in &records {
        scope.core.add_to(index);
        scope.collaboration.add_to(index);
        scope.ideation.add_to(index);
    }
    for scope in &records {
        scope.core.validate(index, &scope.scope_id, dangling);
        scope
            .collaboration
            .validate(index, &scope.scope_id, dangling);
        scope.ideation.validate(index, &scope.scope_id, dangling);
    }

    Ok(())
}
