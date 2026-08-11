mod identity;
mod reconcile;

use std::collections::BTreeMap;

use provenance_core::{EdgeType, NodeType, ScopeId, StableId, SUPPORTED_SCHEMA_VERSION};

use super::{
    ReconcileState, ReconciledResource, StateStore, TypedRequirementInput, TypedRuleInput,
    TypedSpecInput, TypedSpecResult,
};
use crate::shards;
use identity::{
    declaration_ids, requirement_identity, rule_identity, source_identity, validate_ownership,
    validate_references,
};
use reconcile::{reconcile_requirements, reconcile_rules, reconcile_sources};

impl StateStore {
    /// Reconciles one language-owned desired-state document with canonical state.
    ///
    /// Omitted records and relationships are deliberately retained. This first
    /// lifecycle slice creates and updates only records carrying the same
    /// `declared_by` value, so applying one spec cannot take over another
    /// integration's or a human's records.
    pub fn apply_typed_spec(
        &self,
        scope_id: &ScopeId,
        input: TypedSpecInput,
    ) -> anyhow::Result<TypedSpecResult> {
        self.with_repository_publication(|| self.reconcile_typed_spec(scope_id, input))
    }

    fn reconcile_typed_spec(
        &self,
        scope_id: &ScopeId,
        input: TypedSpecInput,
    ) -> anyhow::Result<TypedSpecResult> {
        self.validate_typed_spec(scope_id, &input)?;

        let source_ids = declaration_ids("source", input.sources.iter().map(source_identity))?;
        let requirement_ids = declaration_ids(
            "requirement",
            input.requirements.iter().map(requirement_identity),
        )?;
        let rule_ids = declaration_ids("rule", input.rules.iter().map(rule_identity))?;
        validate_references(
            &input.requirements,
            &input.rules,
            &source_ids,
            &requirement_ids,
        )?;

        let current_sources = self.list_sources(scope_id)?;
        let current_requirements = self.list_requirements(scope_id)?;
        let current_rules = self.list_rules(scope_id)?;
        validate_ownership(
            &input.declared_by,
            &current_sources,
            source_ids.values(),
            |record| (&record.id, record.declared_by.as_deref()),
        )?;
        validate_ownership(
            &input.declared_by,
            &current_requirements,
            requirement_ids.values(),
            |record| (&record.id, record.declared_by.as_deref()),
        )?;
        validate_ownership(
            &input.declared_by,
            &current_rules,
            rule_ids.values(),
            |record| (&record.id, record.declared_by.as_deref()),
        )?;

        let requirement_relationships = input.requirements.clone();
        let rule_relationships = input.rules.clone();
        let (sources, source_resources) = reconcile_sources(
            current_sources,
            scope_id,
            &input.declared_by,
            input.sources,
            &source_ids,
        )?;
        let (requirements, requirement_resources) = reconcile_requirements(
            current_requirements,
            scope_id,
            &input.declared_by,
            input.requirements,
            &requirement_ids,
            &source_ids,
        );
        let (rules, rule_resources) = reconcile_rules(
            current_rules,
            scope_id,
            &input.declared_by,
            input.rules,
            &rule_ids,
        );

        replace_records(self, &shards::sources_path(&self.layout, scope_id), sources)?;
        replace_records(
            self,
            &shards::requirements_path(&self.layout, scope_id),
            requirements,
        )?;
        replace_records(self, &shards::rules_path(&self.layout, scope_id), rules)?;
        self.write_typed_spec_edges(
            scope_id,
            &requirement_relationships,
            &rule_relationships,
            &source_ids,
            &requirement_ids,
            &rule_ids,
        )?;

        Ok(spec_result(
            input.declared_by,
            source_resources,
            requirement_resources,
            rule_resources,
        ))
    }

    fn validate_typed_spec(
        &self,
        scope_id: &ScopeId,
        input: &TypedSpecInput,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            input.schema_version == SUPPORTED_SCHEMA_VERSION.0,
            "typed spec schema_version must be {}",
            SUPPORTED_SCHEMA_VERSION.0
        );
        anyhow::ensure!(
            !input.declared_by.trim().is_empty(),
            "declared_by must not be empty"
        );
        anyhow::ensure!(
            self.manifest()?
                .scopes
                .iter()
                .any(|scope| scope.id.as_str() == scope_id.as_str()),
            "scope `{}` does not exist",
            scope_id.as_str()
        );
        Ok(())
    }

    fn write_typed_spec_edges(
        &self,
        scope_id: &ScopeId,
        requirements: &[TypedRequirementInput],
        rules: &[TypedRuleInput],
        source_ids: &BTreeMap<String, StableId>,
        requirement_ids: &BTreeMap<String, StableId>,
        rule_ids: &BTreeMap<String, StableId>,
    ) -> anyhow::Result<()> {
        for declaration in requirements {
            for source in &declaration.sources {
                self.add_edge(
                    scope_id.clone(),
                    EdgeType::References,
                    NodeType::Source,
                    source_ids[source].clone(),
                    NodeType::Requirement,
                    requirement_ids[&declaration.key].clone(),
                )?;
            }
        }

        // Relationships are additive in this POC. Reapplying is idempotent,
        // while omission never erases a relationship another owner may use.
        for declaration in rules {
            self.add_edge(
                scope_id.clone(),
                EdgeType::Produces,
                NodeType::Requirement,
                requirement_ids[&declaration.requirement].clone(),
                NodeType::Rule,
                rule_ids[&declaration.key].clone(),
            )?;
        }
        Ok(())
    }
}

fn spec_result(
    declared_by: String,
    source_resources: Vec<ReconciledResource>,
    requirement_resources: Vec<ReconciledResource>,
    rule_resources: Vec<ReconciledResource>,
) -> TypedSpecResult {
    let mut resources = source_resources;
    resources.extend(requirement_resources);
    resources.extend(rule_resources);
    TypedSpecResult {
        declared_by,
        created: count_state(&resources, ReconcileState::Created),
        updated: count_state(&resources, ReconcileState::Updated),
        unchanged: count_state(&resources, ReconcileState::Unchanged),
        resources,
    }
}

fn replace_records<T: serde::de::DeserializeOwned + serde::Serialize>(
    store: &StateStore,
    path: &camino::Utf8Path,
    replacement: Vec<T>,
) -> anyhow::Result<()> {
    store.mutate_jsonl_records(path, |records| {
        *records = replacement;
        Ok(())
    })
}

fn count_state(resources: &[ReconciledResource], state: ReconcileState) -> usize {
    resources
        .iter()
        .filter(|resource| resource.state == state)
        .count()
}
