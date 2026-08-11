use std::collections::{BTreeMap, BTreeSet};

use provenance_core::StableId;
use sha2::{Digest, Sha256};

use super::super::{TypedRequirementInput, TypedRuleInput, TypedSourceInput};

pub(super) fn source_identity(input: &TypedSourceInput) -> (&str, Option<&str>) {
    (&input.key, input.id.as_deref())
}

pub(super) fn requirement_identity(input: &TypedRequirementInput) -> (&str, Option<&str>) {
    (&input.key, input.id.as_deref())
}

pub(super) fn rule_identity(input: &TypedRuleInput) -> (&str, Option<&str>) {
    (&input.key, input.id.as_deref())
}

pub(super) fn declaration_ids<'a>(
    kind: &str,
    declarations: impl Iterator<Item = (&'a str, Option<&'a str>)>,
) -> anyhow::Result<BTreeMap<String, StableId>> {
    let mut ids = BTreeMap::new();
    let mut canonical = BTreeSet::new();
    for (key, explicit_id) in declarations {
        anyhow::ensure!(!key.trim().is_empty(), "{kind} key must not be empty");
        anyhow::ensure!(!ids.contains_key(key), "duplicate {kind} key `{key}`");
        let id = canonical_id(kind, key, explicit_id)?;
        anyhow::ensure!(
            canonical.insert(id.as_str().to_string()),
            "two {kind} declarations resolve to id `{}`",
            id.as_str()
        );
        ids.insert(key.to_string(), id);
    }
    Ok(ids)
}

fn canonical_id(kind: &str, key: &str, explicit_id: Option<&str>) -> anyhow::Result<StableId> {
    if let Some(id) = explicit_id {
        return StableId::new(id);
    }
    if let Ok(id) = StableId::new(key) {
        return Ok(id);
    }
    let slug = key
        .chars()
        .map(|character| match character {
            character if character.is_ascii_alphanumeric() => character.to_ascii_lowercase(),
            '-' | '_' => character,
            _ => '_',
        })
        .collect::<String>();
    let digest = format!("{:x}", Sha256::digest(key.as_bytes()));
    StableId::new(format!("{kind}_{slug}_{}", &digest[..10]))
}

pub(super) fn validate_references(
    requirements: &[TypedRequirementInput],
    rules: &[TypedRuleInput],
    source_ids: &BTreeMap<String, StableId>,
    requirement_ids: &BTreeMap<String, StableId>,
) -> anyhow::Result<()> {
    for requirement in requirements {
        for source in &requirement.sources {
            anyhow::ensure!(
                source_ids.contains_key(source),
                "requirement `{}` references undeclared source `{source}`",
                requirement.key
            );
        }
    }
    for rule in rules {
        anyhow::ensure!(
            requirement_ids.contains_key(&rule.requirement),
            "rule `{}` references undeclared requirement `{}`",
            rule.key,
            rule.requirement
        );
    }
    Ok(())
}

pub(super) fn validate_ownership<'a, T: 'a>(
    owner: &str,
    records: &'a [T],
    desired_ids: impl Iterator<Item = &'a StableId>,
    fields: impl Fn(&'a T) -> (&'a StableId, Option<&'a str>),
) -> anyhow::Result<()> {
    for desired_id in desired_ids {
        let Some(record) = records.iter().find(|record| fields(record).0 == desired_id) else {
            continue;
        };
        let (_, declared_by) = fields(record);
        anyhow::ensure!(
            declared_by == Some(owner),
            "record `{}` is not owned by `{owner}` (declared_by: {})",
            desired_id.as_str(),
            declared_by.unwrap_or("unowned")
        );
    }
    Ok(())
}
