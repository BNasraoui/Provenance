use camino::Utf8Path;
use provenance_core::{Requirement, Rule, ScopeId};
use provenance_macros::rule;
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::StateStore,
    statement_analysis::{analyze_changed_statements, StatementDiagnostic},
};
use serde::de::DeserializeOwned;

#[rule("rule_ste_manual_changed_statement_report")]
pub(super) fn changed_statements_from_head(
    repo: &Utf8Path,
) -> anyhow::Result<Vec<StatementDiagnostic>> {
    if !has_head(repo)? {
        return Ok(Vec::new());
    }

    let store = StateStore::new(ProvenanceLayout::new(repo));
    let manifest = store.manifest()?;
    let mut base_requirements = Vec::new();
    let mut base_rules = Vec::new();
    let mut candidate_requirements = Vec::new();
    let mut candidate_rules = Vec::new();
    for scope in manifest.scopes {
        base_requirements.extend(read_head_records::<Requirement>(
            repo,
            &scope.id,
            "requirements/req.jsonl",
        )?);
        base_rules.extend(read_head_records::<Rule>(
            repo,
            &scope.id,
            "rules/rule.jsonl",
        )?);
        candidate_requirements.extend(store.list_requirements(&scope.id)?);
        candidate_rules.extend(store.list_rules(&scope.id)?);
    }
    Ok(analyze_changed_statements(
        &base_requirements,
        &base_rules,
        &candidate_requirements,
        &candidate_rules,
    ))
}

fn has_head(repo: &Utf8Path) -> anyhow::Result<bool> {
    Ok(std::process::Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "--verify", "HEAD"])
        .output()?
        .status
        .success())
}

fn read_head_records<T: DeserializeOwned>(
    repo: &Utf8Path,
    scope: &ScopeId,
    family_path: &str,
) -> anyhow::Result<Vec<T>> {
    let path = format!(
        ".provenance/state/scopes/{}/{}",
        scope.as_str(),
        family_path
    );
    let object = format!("HEAD:{path}");
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(["show", &object])
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    String::from_utf8(output.stdout)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}
