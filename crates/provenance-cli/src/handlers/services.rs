use crate::cli::services::{ServiceCreateArgs, ServicesCommand};
use crate::output;
use provenance_core::{ScopeId, ServiceEnvironment, ServiceStatus, ServiceTier, StableId};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateServiceInput, StateStore},
};

pub(super) fn handle(command: ServicesCommand) -> anyhow::Result<()> {
    match command {
        ServicesCommand::Create(args) => {
            let ServiceCreateArgs {
                repo,
                scope,
                id,
                name,
                description,
                owner,
                repository,
                environment,
                tier,
                external_id,
                status,
                format,
            } = *args;
            let service = StateStore::new(ProvenanceLayout::new(repo)).create_service(
                CreateServiceInput {
                    scope_id: ScopeId::new(scope)?,
                    id: StableId::new(id)?,
                    name,
                    description,
                    owner,
                    repository,
                    environment: environment
                        .map(|value| ServiceEnvironment::parse(&value))
                        .transpose()?,
                    tier: tier.map(|value| ServiceTier::parse(&value)).transpose()?,
                    external_id,
                    status: ServiceStatus::parse(&status)?,
                },
            )?;
            output::print(format, &service)?;
        }
        ServicesCommand::List {
            repo,
            scope,
            format,
        } => {
            let services = StateStore::new(ProvenanceLayout::new(repo))
                .list_services(&ScopeId::new(scope)?)?;
            output::print(format, &services)?;
        }
    }
    Ok(())
}
