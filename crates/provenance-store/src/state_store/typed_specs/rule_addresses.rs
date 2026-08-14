use std::collections::BTreeMap;

use provenance_core::{DeclarationAddress, StableId};

use super::super::TypedRuleInput;

pub(in crate::state_store) fn rule_address(
    spec: &str,
    declaration: &TypedRuleInput,
) -> anyhow::Result<DeclarationAddress> {
    let inferred = inferred_address(spec, &declaration.requirements, &declaration.key)?;
    let Some(explicit) = &declaration.address else {
        return Ok(inferred);
    };
    let shared = shared_address(spec, &declaration.key)?;
    if explicit == &shared {
        return Ok(explicit.clone());
    }
    if let [requirement] = declaration.requirements.as_slice() {
        let local = local_address(spec, requirement, &declaration.key)?;
        anyhow::ensure!(
            explicit == &local,
            "rule `{}` address must be `{}` or `{}` for requirement `{requirement}`",
            declaration.key,
            shared.segments().join("/"),
            local.segments().join("/")
        );
        return Ok(explicit.clone());
    }
    anyhow::bail!(
        "rule `{}` with several requirements must use shared address `{}`",
        declaration.key,
        shared.segments().join("/")
    )
}

pub(super) fn migration_candidates(
    spec: &str,
    declaration: &TypedRuleInput,
    desired: &DeclarationAddress,
    existing: &BTreeMap<DeclarationAddress, StableId>,
) -> anyhow::Result<Vec<StableId>> {
    let shared = shared_address(spec, &declaration.key)?;
    let addresses = if desired == &shared {
        declaration
            .requirements
            .iter()
            .map(|requirement| local_address(spec, requirement, &declaration.key))
            .collect::<anyhow::Result<Vec<_>>>()?
    } else {
        vec![shared]
    };
    let candidates = addresses
        .iter()
        .filter_map(|address| existing.get(address))
        .map(|id| (id.as_str(), id))
        .collect::<BTreeMap<_, _>>();
    Ok(candidates.into_values().cloned().collect())
}

pub(super) fn local_parent(address: &DeclarationAddress) -> Option<String> {
    match address.segments() {
        [_, kind, requirement, rule, _] if kind == "requirement" && rule == "rule" => {
            Some(requirement.clone())
        }
        _ => None,
    }
}

fn inferred_address(
    spec: &str,
    requirements: &[String],
    key: &str,
) -> anyhow::Result<DeclarationAddress> {
    match requirements {
        [requirement] => local_address(spec, requirement, key),
        [] => anyhow::bail!("rule `{key}` must refine at least one requirement"),
        _ => shared_address(spec, key),
    }
}

fn shared_address(spec: &str, key: &str) -> anyhow::Result<DeclarationAddress> {
    DeclarationAddress::new([spec, "rule", key])
}

fn local_address(spec: &str, requirement: &str, key: &str) -> anyhow::Result<DeclarationAddress> {
    DeclarationAddress::new([spec, "requirement", requirement, "rule", key])
}
