use std::collections::BTreeMap;

use provenance_core::{ImplementationBinding, ScopeId, StableId, SUPPORTED_SCHEMA_VERSION};
use sha2::{Digest, Sha256};

use super::{MaterializeImplementationBindingInput, StateStore, TypedRuleInput};
use crate::shards;

pub(super) fn reconcile(
    store: &StateStore,
    scope_id: &ScopeId,
    owner: &str,
    spec: &str,
    rules: &[TypedRuleInput],
    rule_ids: &BTreeMap<provenance_core::DeclarationAddress, StableId>,
    write: bool,
) -> anyhow::Result<Vec<ImplementationBinding>> {
    let mut existing = store.list_implementation_bindings(scope_id)?;
    let mut desired = Vec::new();
    for rule in rules.iter().filter(|rule| rule.implementation.is_some()) {
        let address = super::typed_specs::rule_address(spec, &rule.requirements, &rule.key)?;
        let rule_id = rule_ids[&address].clone();
        let implementation = rule.implementation.as_ref().unwrap();
        validate_target(&implementation.file, &implementation.symbol)?;
        let id = binding_id(&rule_id)?;
        if let Some(record) = existing.iter().find(|record| record.rule_id == rule_id) {
            anyhow::ensure!(
                record.declared_by == owner,
                "rule `{}` implementation is owned by `{}`",
                rule_id.as_str(),
                record.declared_by
            );
        }
        desired.push(ImplementationBinding {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            scope_id: scope_id.clone(),
            id,
            rule_id,
            declared_by: owner.to_string(),
            file: implementation.file.clone(),
            symbol: implementation.symbol.clone(),
        });
    }
    for binding in &desired {
        if let Some(record) = existing
            .iter_mut()
            .find(|record| record.rule_id == binding.rule_id)
        {
            *record = binding.clone();
        } else {
            existing.push(binding.clone());
        }
    }
    if write {
        existing.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        let path = shards::implementation_bindings_path(&store.layout, scope_id);
        store.mutate_jsonl_records(&path, |records| {
            *records = existing;
            Ok(())
        })?;
    }
    Ok(desired)
}

impl StateStore {
    pub fn materialize_implementation_binding(
        &self,
        input: MaterializeImplementationBindingInput,
    ) -> anyhow::Result<ImplementationBinding> {
        self.with_repository_publication(|| {
            anyhow::ensure!(
                !input.declared_by.trim().is_empty(),
                "declared_by must not be empty"
            );
            validate_target(&input.file, &input.symbol)?;
            anyhow::ensure!(
                self.layout.root().join(&input.file).is_file(),
                "implementation file `{}` does not exist",
                input.file
            );
            anyhow::ensure!(
                self.list_rules(&input.scope_id)?
                    .iter()
                    .any(|rule| rule.id == input.rule_id),
                "rule `{}` does not exist",
                input.rule_id.as_str()
            );
            let id = binding_id(&input.rule_id)?;
            let binding = ImplementationBinding {
                schema_version: SUPPORTED_SCHEMA_VERSION,
                scope_id: input.scope_id.clone(),
                id,
                rule_id: input.rule_id,
                declared_by: input.declared_by,
                file: input.file,
                symbol: input.symbol,
            };
            let path = shards::implementation_bindings_path(&self.layout, &input.scope_id);
            self.mutate_jsonl_records(&path, |records: &mut Vec<ImplementationBinding>| {
                anyhow::ensure!(
                    !records.iter().any(|record| {
                        record.rule_id == binding.rule_id
                            && record.declared_by != binding.declared_by
                    }),
                    "rule `{}` implementation is owned by another declaration owner",
                    binding.rule_id.as_str()
                );
                if let Some(record) = records
                    .iter_mut()
                    .find(|record| record.rule_id == binding.rule_id)
                {
                    *record = binding.clone();
                } else {
                    records.push(binding.clone());
                }
                records.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
                Ok(binding)
            })
        })
    }
}

fn validate_target(file: &camino::Utf8Path, symbol: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !file.as_str().is_empty(),
        "implementation file must not be empty"
    );
    anyhow::ensure!(
        !file.is_absolute()
            && !file
                .components()
                .any(|part| matches!(part, camino::Utf8Component::ParentDir)),
        "implementation file must be a repository-relative path"
    );
    anyhow::ensure!(
        !symbol.trim().is_empty(),
        "implementation symbol must not be empty"
    );
    Ok(())
}

fn binding_id(rule_id: &StableId) -> anyhow::Result<StableId> {
    let digest = format!("{:x}", Sha256::digest(rule_id.as_str().as_bytes()));
    StableId::new(format!("implementation_binding_{}", &digest[..20]))
}
