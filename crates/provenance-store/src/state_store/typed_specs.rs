mod identity;
mod reconcile;
mod rule_addresses;

use std::collections::BTreeMap;

use provenance_core::{
    DeclarationAddress, EdgeType, NodeType, Requirement, Rule, ScopeId, Source, StableId,
    SUPPORTED_SCHEMA_VERSION,
};

use super::{
    ReconcileState, ReconciledResource, StateStore, TypedRequirementInput, TypedRuleInput,
    TypedSpecInput, TypedSpecResult,
};
use crate::shards;
use identity::{
    declaration_ids, normalize_rule_relationships, owned_declaration_ids, requirement_identity,
    rule_declaration_ids, source_identity, validate_ownership, validate_references,
};
use reconcile::{reconcile_requirements, reconcile_rules, reconcile_sources};
pub(in crate::state_store) use rule_addresses::rule_address;

struct CurrentTypedState {
    sources: Vec<Source>,
    requirements: Vec<Requirement>,
    rules: Vec<Rule>,
    source_addresses: BTreeMap<DeclarationAddress, StableId>,
    requirement_addresses: BTreeMap<DeclarationAddress, StableId>,
    rule_addresses: BTreeMap<DeclarationAddress, StableId>,
}

struct DesiredTypedIds {
    sources: BTreeMap<String, StableId>,
    requirements: BTreeMap<String, StableId>,
    rules: BTreeMap<DeclarationAddress, StableId>,
}

#[derive(Clone, Copy)]
struct DesiredTypedGraph<'a> {
    spec: &'a str,
    requirements: &'a [TypedRequirementInput],
    rules: &'a [TypedRuleInput],
    source_ids: &'a BTreeMap<String, StableId>,
    requirement_ids: &'a BTreeMap<String, StableId>,
    rule_ids: &'a BTreeMap<DeclarationAddress, StableId>,
}

#[derive(Clone, Copy)]
enum ReconcileMode {
    Plan,
    Apply,
}

fn desired_typed_ids(
    input: &TypedSpecInput,
    current: &CurrentTypedState,
) -> anyhow::Result<DesiredTypedIds> {
    let sources = declaration_ids(
        "source",
        &input.declared_by,
        &input.spec,
        input.sources.iter().map(source_identity),
        &current.source_addresses,
    )?;
    let requirements = declaration_ids(
        "requirement",
        &input.declared_by,
        &input.spec,
        input.requirements.iter().map(requirement_identity),
        &current.requirement_addresses,
    )?;
    let rules = rule_declaration_ids(
        &input.declared_by,
        &input.spec,
        &input.rules,
        &current.rule_addresses,
    )?;
    validate_references(&input.requirements, &input.rules, &sources, &requirements)?;
    validate_ownership(
        &input.declared_by,
        &current.sources,
        sources.values(),
        |record| (&record.id, record.declared_by.as_deref()),
    )?;
    validate_ownership(
        &input.declared_by,
        &current.requirements,
        requirements.values(),
        |record| (&record.id, record.declared_by.as_deref()),
    )?;
    validate_ownership(
        &input.declared_by,
        &current.rules,
        rules.values(),
        |record| (&record.id, record.declared_by.as_deref()),
    )?;
    Ok(DesiredTypedIds {
        sources,
        requirements,
        rules,
    })
}

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
        self.with_repository_publication(|| {
            self.reconcile_typed_spec(scope_id, input, ReconcileMode::Apply)
        })
    }

    /// Calculates the exact typed-spec reconciliation without publishing it.
    pub fn plan_typed_spec(
        &self,
        scope_id: &ScopeId,
        input: TypedSpecInput,
    ) -> anyhow::Result<TypedSpecResult> {
        self.with_repository_publication(|| {
            self.reconcile_typed_spec(scope_id, input, ReconcileMode::Plan)
        })
    }

    fn reconcile_typed_spec(
        &self,
        scope_id: &ScopeId,
        input: TypedSpecInput,
        mode: ReconcileMode,
    ) -> anyhow::Result<TypedSpecResult> {
        let (input, current) = self.prepare_typed_spec(scope_id, input)?;
        let ids = desired_typed_ids(&input, &current)?;

        let requirement_relationships = input.requirements.clone();
        let rule_relationships = input.rules.clone();
        let spec = input.spec;
        let (sources, source_resources) = reconcile_sources(
            current.sources,
            &spec,
            scope_id,
            &input.declared_by,
            input.sources,
            &ids.sources,
        )?;
        let (requirements, requirement_resources) = reconcile_requirements(
            current.requirements,
            &spec,
            scope_id,
            &input.declared_by,
            input.requirements,
            &ids.requirements,
            &ids.sources,
        )?;
        let (rules, rule_resources) = reconcile_rules(
            current.rules,
            &spec,
            scope_id,
            &input.declared_by,
            input.rules,
            &ids.rules,
        )?;
        let implementation_bindings = super::implementation_bindings::reconcile(
            self,
            scope_id,
            &input.declared_by,
            &spec,
            &rule_relationships,
            &ids.rules,
            false,
        )?;
        let mut result = spec_result(
            input.declared_by.clone(),
            source_resources,
            requirement_resources,
            rule_resources,
            implementation_bindings,
        );
        let dictionary = crate::dictionary_reference::load_project_dictionary(&self.layout);
        result.diagnostics = super::typed_statement_policy::analyze_typed_statements(
            &result.resources,
            &requirements,
            &rules,
            dictionary.as_ref(),
        );

        if matches!(mode, ReconcileMode::Apply) {
            super::typed_statement_policy::ensure_typed_spec_is_writable(&result)?;
            replace_records(self, &shards::sources_path(&self.layout, scope_id), sources)?;
            replace_records(
                self,
                &shards::requirements_path(&self.layout, scope_id),
                requirements,
            )?;
            replace_records(self, &shards::rules_path(&self.layout, scope_id), rules)?;
            super::implementation_bindings::reconcile(
                self,
                scope_id,
                &input.declared_by,
                &spec,
                &rule_relationships,
                &ids.rules,
                true,
            )?;
            self.write_typed_spec_edges(
                scope_id,
                DesiredTypedGraph {
                    spec: &spec,
                    requirements: &requirement_relationships,
                    rules: &rule_relationships,
                    source_ids: &ids.sources,
                    requirement_ids: &ids.requirements,
                    rule_ids: &ids.rules,
                },
            )?;
        }

        Ok(result)
    }

    fn current_typed_state(
        &self,
        scope_id: &ScopeId,
        owner: &str,
    ) -> anyhow::Result<CurrentTypedState> {
        let sources = self.list_sources(scope_id)?;
        let requirements = self.list_requirements(scope_id)?;
        let rules = self.list_rules(scope_id)?;
        let source_addresses = owned_declaration_ids(owner, &sources, |record| {
            (
                &record.id,
                record.declared_by.as_deref(),
                record.declaration_address.as_ref(),
            )
        })?;
        let requirement_addresses = owned_declaration_ids(owner, &requirements, |record| {
            (
                &record.id,
                record.declared_by.as_deref(),
                record.declaration_address.as_ref(),
            )
        })?;
        let rule_addresses = owned_declaration_ids(owner, &rules, |record| {
            (
                &record.id,
                record.declared_by.as_deref(),
                record.declaration_address.as_ref(),
            )
        })?;
        Ok(CurrentTypedState {
            sources,
            requirements,
            rules,
            source_addresses,
            requirement_addresses,
            rule_addresses,
        })
    }

    fn prepare_typed_spec(
        &self,
        scope_id: &ScopeId,
        mut input: TypedSpecInput,
    ) -> anyhow::Result<(TypedSpecInput, CurrentTypedState)> {
        self.validate_typed_spec(scope_id, &input)?;
        normalize_rule_relationships(&mut input.rules)?;
        let current = self.current_typed_state(scope_id, &input.declared_by)?;
        Ok((input, current))
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
        anyhow::ensure!(!input.spec.trim().is_empty(), "spec must not be empty");
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
        graph: DesiredTypedGraph<'_>,
    ) -> anyhow::Result<()> {
        for declaration in graph.requirements {
            for source in &declaration.sources {
                self.add_edge(
                    scope_id.clone(),
                    EdgeType::References,
                    NodeType::Source,
                    graph.source_ids[source].clone(),
                    NodeType::Requirement,
                    graph.requirement_ids[&declaration.key].clone(),
                )?;
            }
        }

        // Relationships are additive in this POC. Reapplying is idempotent,
        // while omission never erases a relationship another owner may use.
        for declaration in graph.rules {
            let address = rule_address(graph.spec, declaration)?;
            for requirement in &declaration.requirements {
                self.add_edge(
                    scope_id.clone(),
                    EdgeType::Produces,
                    NodeType::Requirement,
                    graph.requirement_ids[requirement].clone(),
                    NodeType::Rule,
                    graph.rule_ids[&address].clone(),
                )?;
            }
        }
        Ok(())
    }
}

fn spec_result(
    declared_by: String,
    source_resources: Vec<ReconciledResource>,
    requirement_resources: Vec<ReconciledResource>,
    rule_resources: Vec<ReconciledResource>,
    implementation_bindings: Vec<provenance_core::ImplementationBinding>,
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
        diagnostics: Vec::new(),
        implementation_bindings,
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
