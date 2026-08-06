use super::{CreateDomainInput, CreateServiceBindingInput, CreateServiceInput, StateStore};
use crate::shards;
use provenance_core::{Domain, SchemaVersion, Service, ServiceBinding};

impl StateStore {
    pub fn create_domain(&self, input: CreateDomainInput) -> anyhow::Result<Domain> {
        let CreateDomainInput {
            scope_id,
            id,
            name,
            description,
            color,
        } = input;
        let path = shards::domains_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<Domain>| {
            let domain = Domain {
                schema_version: SchemaVersion(1),
                scope_id: scope_id.clone(),
                id,
                name,
                description,
                color,
            };
            anyhow::ensure!(
                !records.iter().any(|record| record.id == domain.id),
                "domain already exists"
            );
            anyhow::ensure!(
                !records.iter().any(|record| record.name == domain.name),
                "domain name already exists"
            );
            records.push(domain.clone());
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(domain)
        })
    }

    pub fn create_service(&self, input: CreateServiceInput) -> anyhow::Result<Service> {
        let CreateServiceInput {
            scope_id,
            id,
            name,
            description,
            owner,
            repository,
            environment,
            tier,
            external_id,
            status,
        } = input;
        let path = shards::services_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<Service>| {
            let service = Service {
                schema_version: SchemaVersion(1),
                scope_id: scope_id.clone(),
                id,
                name,
                description,
                owner,
                repository,
                environment,
                tier,
                external_id,
                status,
            };
            anyhow::ensure!(
                !records.iter().any(|record| record.id == service.id),
                "service already exists"
            );
            anyhow::ensure!(
                !records.iter().any(|record| record.name == service.name),
                "service name already exists"
            );
            records.push(service.clone());
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(service)
        })
    }

    pub fn create_service_binding(
        &self,
        input: CreateServiceBindingInput,
    ) -> anyhow::Result<ServiceBinding> {
        self.with_repository_publication(|| self.write_service_binding(input))
    }

    fn write_service_binding(
        &self,
        input: CreateServiceBindingInput,
    ) -> anyhow::Result<ServiceBinding> {
        let CreateServiceBindingInput {
            scope_id,
            rule_id,
            service_id,
            binding_type,
        } = input;
        anyhow::ensure!(
            self.list_rules(&scope_id)?
                .iter()
                .any(|rule| rule.id == rule_id),
            "rule does not exist"
        );
        anyhow::ensure!(
            self.list_services(&scope_id)?
                .iter()
                .any(|service| service.id == service_id),
            "service does not exist"
        );
        let path = shards::service_bindings_path(&self.layout, &scope_id);
        self.mutate_jsonl_records(&path, |records: &mut Vec<ServiceBinding>| {
            let binding = ServiceBinding {
                schema_version: SchemaVersion(1),
                scope_id: scope_id.clone(),
                id: ServiceBinding::stable_id(&rule_id, &service_id, binding_type)?,
                rule_id,
                service_id,
                binding_type,
            };
            anyhow::ensure!(
                !records.iter().any(|record| record.id == binding.id),
                "service binding already exists"
            );
            records.push(binding.clone());
            records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(binding)
        })
    }
}
