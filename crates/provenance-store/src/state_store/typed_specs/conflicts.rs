use provenance_core::{DeclarationAddress, Requirement, Rule, Source, StableId};

use super::{rule_address, CurrentTypedState, DesiredTypedIds};
use crate::state_store::{
    ReconcileState, ReconciledResource, TypedFieldChange, TypedResourceKind, TypedSpecInput,
};

pub(super) fn ownership_conflicts(
    input: &TypedSpecInput,
    current: &CurrentTypedState,
    ids: &DesiredTypedIds,
) -> anyhow::Result<Vec<ReconciledResource>> {
    let mut conflicts = Vec::new();
    for declaration in &input.sources {
        if let Some(owner) =
            foreign_source_owner(&current.sources, &ids.sources[&declaration.key], input)
        {
            conflicts.push(conflict(
                TypedResourceKind::Source,
                &declaration.key,
                None,
                DeclarationAddress::new([input.spec.as_str(), "source", &declaration.key])?,
                ids.sources[&declaration.key].clone(),
                owner,
                &input.declared_by,
            ));
        }
    }
    for declaration in &input.requirements {
        if let Some(owner) = foreign_requirement_owner(
            &current.requirements,
            &ids.requirements[&declaration.key],
            input,
        ) {
            conflicts.push(conflict(
                TypedResourceKind::Requirement,
                &declaration.key,
                None,
                DeclarationAddress::new([input.spec.as_str(), "requirement", &declaration.key])?,
                ids.requirements[&declaration.key].clone(),
                owner,
                &input.declared_by,
            ));
        }
    }
    for declaration in &input.rules {
        let address = rule_address(&input.spec, declaration)?;
        let id = &ids.rules[&address];
        if let Some(owner) = foreign_rule_owner(&current.rules, id, input) {
            conflicts.push(conflict(
                TypedResourceKind::Rule,
                &declaration.key,
                super::rule_addresses::local_parent(&address),
                address,
                id.clone(),
                owner,
                &input.declared_by,
            ));
        }
    }
    Ok(conflicts)
}

fn foreign_source_owner<'a>(
    records: &'a [Source],
    id: &StableId,
    input: &TypedSpecInput,
) -> Option<&'a str> {
    foreign_owner(records, id, &input.declared_by, |record| {
        (&record.id, record.declared_by.as_deref())
    })
}

fn foreign_requirement_owner<'a>(
    records: &'a [Requirement],
    id: &StableId,
    input: &TypedSpecInput,
) -> Option<&'a str> {
    foreign_owner(records, id, &input.declared_by, |record| {
        (&record.id, record.declared_by.as_deref())
    })
}

fn foreign_rule_owner<'a>(
    records: &'a [Rule],
    id: &StableId,
    input: &TypedSpecInput,
) -> Option<&'a str> {
    foreign_owner(records, id, &input.declared_by, |record| {
        (&record.id, record.declared_by.as_deref())
    })
}

fn foreign_owner<'a, T>(
    records: &'a [T],
    id: &StableId,
    desired_owner: &str,
    fields: impl Fn(&'a T) -> (&'a StableId, Option<&'a str>),
) -> Option<&'a str> {
    let record = records.iter().find(|record| fields(record).0 == id)?;
    let owner = fields(record).1.unwrap_or("unowned");
    (owner != desired_owner).then_some(owner)
}

fn conflict(
    kind: TypedResourceKind,
    key: &str,
    parent: Option<String>,
    address: DeclarationAddress,
    id: StableId,
    current_owner: &str,
    desired_owner: &str,
) -> ReconciledResource {
    ReconciledResource {
        kind,
        key: key.to_string(),
        parent,
        address,
        id,
        state: ReconcileState::Conflict,
        changes: vec![TypedFieldChange {
            field: "declared_by".to_string(),
            before: serde_json::Value::String(current_owner.to_string()),
            after: serde_json::Value::String(desired_owner.to_string()),
        }],
    }
}
