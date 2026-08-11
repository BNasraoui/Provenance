use std::collections::BTreeMap;

use provenance_core::{
    Requirement, RequirementStatus, Rule, RuleSeverity, RuleStatus, ScopeId, Source,
    SourceReference, SourceType, StableId, SUPPORTED_SCHEMA_VERSION,
};

use super::super::{
    ReconcileState, ReconciledResource, TypedRequirementInput, TypedResourceKind, TypedRuleInput,
    TypedSourceInput,
};

pub(super) fn reconcile_sources(
    mut records: Vec<Source>,
    scope_id: &ScopeId,
    owner: &str,
    declarations: Vec<TypedSourceInput>,
    ids: &BTreeMap<String, StableId>,
) -> anyhow::Result<(Vec<Source>, Vec<ReconciledResource>)> {
    let mut resources = Vec::new();
    for declaration in declarations {
        let id = ids[&declaration.key].clone();
        let source_type = source_type(&declaration.kind)?;
        let desired = Source {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            scope_id: scope_id.clone(),
            id: id.clone(),
            declared_by: Some(owner.to_string()),
            name: declaration.name,
            source_type,
            url: declaration.url,
            reference: declaration.reference,
            commit_pin: None,
            effective_date: None,
            review_date: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        };
        let state = upsert_source(&mut records, desired);
        resources.push(resource(
            TypedResourceKind::Source,
            declaration.key,
            id,
            state,
        ));
    }
    records.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    Ok((records, resources))
}

fn source_type(kind: &str) -> anyhow::Result<SourceType> {
    SourceType::parse(kind).or_else(|_| match kind.to_ascii_lowercase().as_str() {
        "linear" | "github" | "jira" => Ok(SourceType::ExternalIntegration),
        _ => anyhow::bail!("source kind `{kind}` is not supported"),
    })
}

fn upsert_source(records: &mut Vec<Source>, desired: Source) -> ReconcileState {
    let Some(existing) = records.iter_mut().find(|record| record.id == desired.id) else {
        records.push(desired);
        return ReconcileState::Created;
    };
    let before = existing.clone();
    existing.declared_by = desired.declared_by;
    existing.name = desired.name;
    existing.source_type = desired.source_type;
    existing.url = desired.url;
    existing.reference = desired.reference;
    state_after_change(existing, &before)
}

pub(super) fn reconcile_requirements(
    mut records: Vec<Requirement>,
    scope_id: &ScopeId,
    owner: &str,
    declarations: Vec<TypedRequirementInput>,
    ids: &BTreeMap<String, StableId>,
    source_ids: &BTreeMap<String, StableId>,
) -> (Vec<Requirement>, Vec<ReconciledResource>) {
    let mut resources = Vec::new();
    for declaration in declarations {
        let id = ids[&declaration.key].clone();
        let source_refs = declaration
            .sources
            .iter()
            .map(|key| SourceReference {
                source_id: source_ids[key].clone(),
                clause: None,
            })
            .collect::<Vec<_>>();
        let desired = Requirement {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            scope_id: scope_id.clone(),
            id: id.clone(),
            declared_by: Some(owner.to_string()),
            statement: declaration.statement,
            description: declaration.description,
            fog: None,
            status: RequirementStatus::Active,
            domain_id: None,
            source_refs,
            origin_thread: None,
            origin_message: None,
        };
        let state = upsert_requirement(&mut records, desired);
        resources.push(resource(
            TypedResourceKind::Requirement,
            declaration.key,
            id,
            state,
        ));
    }
    records.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    (records, resources)
}

fn upsert_requirement(records: &mut Vec<Requirement>, desired: Requirement) -> ReconcileState {
    let Some(existing) = records.iter_mut().find(|record| record.id == desired.id) else {
        records.push(desired);
        return ReconcileState::Created;
    };
    let before = existing.clone();
    existing.declared_by = desired.declared_by;
    existing.statement = desired.statement;
    if desired.description.is_some() {
        existing.description = desired.description;
    }
    for source in desired.source_refs {
        if !existing.source_refs.contains(&source) {
            existing.source_refs.push(source);
        }
    }
    existing.source_refs.sort_by(|left, right| {
        left.source_id
            .as_str()
            .cmp(right.source_id.as_str())
            .then(left.clause.cmp(&right.clause))
    });
    state_after_change(existing, &before)
}

pub(super) fn reconcile_rules(
    mut records: Vec<Rule>,
    scope_id: &ScopeId,
    owner: &str,
    declarations: Vec<TypedRuleInput>,
    ids: &BTreeMap<String, StableId>,
) -> (Vec<Rule>, Vec<ReconciledResource>) {
    let mut resources = Vec::new();
    for declaration in declarations {
        let id = ids[&declaration.key].clone();
        let desired = Rule {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            scope_id: scope_id.clone(),
            id: id.clone(),
            declared_by: Some(owner.to_string()),
            name: declaration.name,
            description: declaration.description,
            statement: declaration.statement,
            status: RuleStatus::Active,
            severity: RuleSeverity::Medium,
            source_document: None,
            source_section: None,
            origin_thread: None,
            origin_message: None,
        };
        let state = upsert_rule(&mut records, desired);
        resources.push(resource(
            TypedResourceKind::Rule,
            declaration.key,
            id,
            state,
        ));
    }
    records.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    (records, resources)
}

fn upsert_rule(records: &mut Vec<Rule>, desired: Rule) -> ReconcileState {
    let Some(existing) = records.iter_mut().find(|record| record.id == desired.id) else {
        records.push(desired);
        return ReconcileState::Created;
    };
    let before = existing.clone();
    existing.declared_by = desired.declared_by;
    existing.statement = desired.statement;
    if desired.name.is_some() {
        existing.name = desired.name;
    }
    if desired.description.is_some() {
        existing.description = desired.description;
    }
    state_after_change(existing, &before)
}

fn state_after_change<T: PartialEq>(changed: &T, before: &T) -> ReconcileState {
    if changed == before {
        ReconcileState::Unchanged
    } else {
        ReconcileState::Updated
    }
}

const fn resource(
    kind: TypedResourceKind,
    key: String,
    id: StableId,
    state: ReconcileState,
) -> ReconciledResource {
    ReconciledResource {
        kind,
        key,
        id,
        state,
    }
}
