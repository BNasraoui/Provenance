use crate::cli::ServiceBindingsCommand;
use crate::output;
use provenance_core::{ScopeId, ServiceBindingType, StableId};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateServiceBindingInput, StateStore},
};

pub(super) fn handle(command: ServiceBindingsCommand) -> anyhow::Result<()> {
    match command {
        ServiceBindingsCommand::Create {
            repo,
            scope,
            rule_id,
            service_id,
            binding_type,
            format,
        } => {
            let binding = StateStore::new(ProvenanceLayout::new(repo)).create_service_binding(
                CreateServiceBindingInput {
                    scope_id: ScopeId::new(scope)?,
                    rule_id: StableId::new(rule_id)?,
                    service_id: StableId::new(service_id)?,
                    binding_type: ServiceBindingType::parse(&binding_type)?,
                },
            )?;
            output::print(format, &binding)?;
        }
        ServiceBindingsCommand::List {
            repo,
            scope,
            format,
        } => {
            let bindings = StateStore::new(ProvenanceLayout::new(repo))
                .list_service_bindings(&ScopeId::new(scope)?)?;
            output::print(format, &bindings)?;
        }
    }
    Ok(())
}
