use camino::{Utf8Path, Utf8PathBuf};
use provenance_core::{EdgeType, ImplementationBinding, NodeType, StableId, VerificationBinding};
use provenance_scanner::{source_sites, SourceSiteRole};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{ReconcileState, StateStore, TypedResourceKind, TypedSpecInput, TypedSpecResult},
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;

mod evidence;
mod human;

pub(super) use human::render;

#[derive(Serialize)]
pub(super) struct TypedSpecPlan {
    #[serde(flatten)]
    reconciliation: TypedSpecResult,
    affected_rules: Vec<AffectedRule>,
}

#[derive(Serialize)]
pub(super) struct AffectedRule {
    id: StableId,
    implementations: Vec<ImplementationSite>,
    verifications: Vec<VerificationSite>,
    evidence: evidence::RuleEvidence,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize)]
pub(super) struct ImplementationSite {
    file: Utf8PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize)]
pub(super) struct VerificationSite {
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    declared_by: Option<String>,
    file: Utf8PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
}

impl ImplementationSite {
    /// Names where a reader should look, as precisely as the record allows.
    fn location(&self) -> String {
        match (self.line, self.symbol.as_deref()) {
            (Some(line), _) => format!("{}:{line}", self.file),
            (None, Some(symbol)) => format!("{} ({symbol})", self.file),
            (None, None) => self.file.to_string(),
        }
    }
}

impl VerificationSite {
    /// Names the test site and how it checks the Rule.
    fn location(&self) -> String {
        let mut location = self.line.map_or_else(
            || self.file.to_string(),
            |line| format!("{}:{line}", self.file),
        );
        if let Some(symbol) = &self.symbol {
            let _ = write!(location, " ({symbol})");
        }
        self.key.as_ref().map_or_else(
            || format!("{location} [{}]", self.method),
            |key| format!("{location} [{key}, {}]", self.method),
        )
    }
}

pub(super) fn typed_spec(
    repo: &Utf8Path,
    scope: &provenance_core::ScopeId,
    input: TypedSpecInput,
) -> anyhow::Result<TypedSpecPlan> {
    let store = StateStore::new(ProvenanceLayout::new(repo.to_path_buf()));
    let reconciliation = store.plan_typed_spec(scope, input)?;
    let reviews = evidence::reviews(&store, scope, &reconciliation)?;
    let mut changed_rules = affected_rule_ids(&store, scope, &reconciliation)?;
    changed_rules.extend(reviews.rules.iter().cloned());
    changed_rules.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    changed_rules.dedup();
    let scans = provenance_scanner::scan_path(repo)?;
    let bindings = store.active_verification_bindings(scope)?;
    let implementation_changes = reconciliation
        .resources
        .iter()
        .filter(|resource| {
            resource.kind == TypedResourceKind::Rule
                && resource
                    .changes
                    .iter()
                    .any(|change| change.field == "implementation")
        })
        .map(|resource| &resource.id)
        .collect::<Vec<_>>();
    let implementations = store
        .active_implementation_bindings(scope)?
        .into_iter()
        .filter(|binding| !implementation_changes.contains(&&binding.rule_id))
        .chain(reconciliation.implementation_bindings.iter().cloned())
        .map(|binding| (binding.id.as_str().to_string(), binding))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect::<Vec<_>>();
    let affected_rules = changed_rules
        .into_iter()
        .map(|id| {
            let rule_evidence = reviews.evidence(&id);
            affected_rule(repo, id, &scans, &bindings, &implementations, rule_evidence)
        })
        .collect();
    Ok(TypedSpecPlan {
        reconciliation,
        affected_rules,
    })
}

fn affected_rule_ids(
    store: &StateStore,
    scope: &provenance_core::ScopeId,
    reconciliation: &TypedSpecResult,
) -> anyhow::Result<Vec<StableId>> {
    let changed = reconciliation
        .resources
        .iter()
        .filter(|resource| resource.state != ReconcileState::Unchanged)
        .collect::<Vec<_>>();
    let changed_requirements = changed
        .iter()
        .filter(|resource| resource.kind == TypedResourceKind::Requirement)
        .map(|resource| &resource.id)
        .collect::<Vec<_>>();
    let mut rules = changed
        .iter()
        .filter(|resource| resource.kind == TypedResourceKind::Rule)
        .map(|resource| resource.id.clone())
        .collect::<Vec<_>>();
    let existing_implementations = store.active_implementation_bindings(scope)?;
    for binding in &reconciliation.implementation_bindings {
        if !existing_implementations
            .iter()
            .any(|existing| existing == binding)
        {
            push_unique(&mut rules, binding.rule_id.clone());
        }
    }

    for edge in store.list_edges()?.into_iter().filter(|edge| {
        edge.scope_id == *scope
            && edge.edge_type == EdgeType::Produces
            && edge.from_type == NodeType::Requirement
            && edge.to_type == NodeType::Rule
            && changed_requirements.contains(&&edge.from_id)
    }) {
        push_unique(&mut rules, edge.to_id);
    }
    for requirement in changed
        .iter()
        .filter(|resource| resource.kind == TypedResourceKind::Requirement)
    {
        for rule in reconciliation.resources.iter().filter(|resource| {
            resource.kind == TypedResourceKind::Rule
                && resource.parent.as_deref() == Some(requirement.key.as_str())
        }) {
            push_unique(&mut rules, rule.id.clone());
        }
    }
    Ok(rules)
}

fn push_unique(ids: &mut Vec<StableId>, id: StableId) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn affected_rule(
    repo: &Utf8Path,
    id: StableId,
    scans: &[provenance_scanner::FileScan],
    bindings: &[VerificationBinding],
    typed_implementations: &[ImplementationBinding],
    evidence: evidence::RuleEvidence,
) -> AffectedRule {
    let mut implementations = source_sites(scans)
        .filter(|site| site.rule_id() == id.as_str())
        .filter(|site| site.role() == SourceSiteRole::Implementation)
        .map(|site| ImplementationSite {
            file: relative(repo, site.file_path()),
            line: Some(site.line()),
            symbol: None,
        })
        .chain(
            typed_implementations
                .iter()
                .filter(|binding| !binding.retired && binding.rule_id == id)
                .map(|binding| ImplementationSite {
                    file: binding.file.clone(),
                    line: None,
                    symbol: Some(binding.symbol.clone()),
                }),
        )
        .collect::<Vec<_>>();
    implementations.sort();
    implementations.dedup();

    let mut verifications = source_sites(scans)
        .filter(|site| site.rule_id() == id.as_str())
        .filter_map(|site| {
            site.verification().map(|method| VerificationSite {
                key: None,
                method: method.to_string(),
                declared_by: None,
                file: relative(repo, site.file_path()),
                line: Some(site.line()),
                symbol: None,
            })
        })
        .chain(
            bindings
                .iter()
                .filter(|binding| binding.rule_id == id)
                .map(typed_verification),
        )
        .collect::<Vec<_>>();
    verifications.sort();
    verifications.dedup();
    AffectedRule {
        id,
        implementations,
        verifications,
        evidence,
    }
}

fn typed_verification(binding: &VerificationBinding) -> VerificationSite {
    VerificationSite {
        key: Some(binding.key.clone()),
        method: binding.method.to_string(),
        declared_by: Some(binding.declared_by.clone()),
        file: binding.file.clone(),
        line: None,
        symbol: binding.symbol.clone(),
    }
}

fn relative(repo: &Utf8Path, file: &Utf8Path) -> Utf8PathBuf {
    file.strip_prefix(repo)
        .map_or_else(|_| file.to_path_buf(), Utf8Path::to_path_buf)
}
