use super::index::CheckIndex;
use provenance_core::{ScopeId, StableId};
use provenance_store::state_store::ScopeSnapshot;

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
    fn load(mut snapshot: ScopeSnapshot) -> Self {
        let scope_id = snapshot.scope.clone();
        let collaboration = collaboration::Records::load(&mut snapshot);
        let ideation = ideation::Records::load(&mut snapshot);
        Self {
            scope_id,
            collaboration,
            ideation,
            core: core::Records::load(snapshot),
        }
    }

    fn validate_scope_ownership(&self, findings: &mut Vec<String>) {
        self.core.validate_scope_ownership(&self.scope_id, findings);
        self.collaboration
            .validate_scope_ownership(&self.scope_id, findings);
        self.ideation
            .validate_scope_ownership(&self.scope_id, findings);
    }
}

fn check_scope_ownership(
    loaded_scope_id: &ScopeId,
    embedded_scope_id: &ScopeId,
    record_type: &str,
    record_id: &StableId,
    findings: &mut Vec<String>,
) {
    if loaded_scope_id != embedded_scope_id {
        findings.push(format!(
            "{record_type} {} loaded from scope {} has embedded scope_id {}",
            record_id.as_str(),
            loaded_scope_id.as_str(),
            embedded_scope_id.as_str()
        ));
    }
}

pub(super) fn validate(
    snapshots: Vec<ScopeSnapshot>,
    index: &mut CheckIndex,
    dangling: &mut Vec<String>,
) -> anyhow::Result<()> {
    let records = snapshots
        .into_iter()
        .map(ScopeRecords::load)
        .collect::<Vec<_>>();

    let mut ownership_findings = Vec::new();
    for scope in &records {
        scope.validate_scope_ownership(&mut ownership_findings);
    }
    anyhow::ensure!(
        ownership_findings.is_empty(),
        "scope ownership finding(s):\n- {}",
        ownership_findings.join("\n- ")
    );

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
