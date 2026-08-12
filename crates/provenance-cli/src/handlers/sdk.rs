use std::io::Read as _;
use std::str::FromStr as _;

use crate::cli::sdk::SdkCommand;
use crate::output;
use provenance_core::{ScopeId, StableId};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{BeginVerificationInput, CompleteVerificationInput, StateStore, TypedSpecInput},
};

mod plan;

pub(super) fn handle(command: SdkCommand) -> anyhow::Result<()> {
    match command {
        SdkCommand::Plan {
            repo,
            scope,
            format,
        } => {
            let input = read_stdin_json::<TypedSpecInput>()?;
            let result = plan::typed_spec(&repo, &ScopeId::new(scope)?, input)?;
            output::print(format, &result)?;
        }
        SdkCommand::Apply {
            repo,
            scope,
            format,
        } => {
            let input = read_stdin_json::<TypedSpecInput>()?;
            let scope_id = ScopeId::new(scope)?;
            let result =
                StateStore::new(ProvenanceLayout::new(repo)).apply_typed_spec(&scope_id, input)?;
            output::print(format, &result)?;
        }
        SdkCommand::BeginVerification {
            repo,
            scope,
            format,
        } => {
            let mut input = read_stdin_json::<BeginVerificationInput>()?;
            input.method = provenance_scanner::Verification::from_str(&input.method)
                .map_err(anyhow::Error::msg)?
                .to_string();
            normalize_verification_context(&repo, &mut input)?;
            let run = StateStore::new(ProvenanceLayout::new(repo))
                .begin_verification(ScopeId::new(scope)?, input)?;
            output::print(format, &run)?;
        }
        SdkCommand::CompleteVerification {
            repo,
            scope,
            format,
        } => {
            let input = read_stdin_json::<CompleteVerificationInput>()?;
            let run = StateStore::new(ProvenanceLayout::new(repo))
                .complete_verification(&ScopeId::new(scope)?, input)?;
            output::print(format, &run)?;
        }
        SdkCommand::VerificationRuns {
            repo,
            scope,
            rule,
            format,
        } => {
            let rule = rule.map(StableId::new).transpose()?;
            let runs = StateStore::new(ProvenanceLayout::new(repo))
                .list_verification_runs(&ScopeId::new(scope)?)?
                .into_iter()
                .filter(|run| rule.as_ref().is_none_or(|rule| &run.rule_id == rule))
                .collect::<Vec<_>>();
            output::print(format, &runs)?;
        }
        SdkCommand::VerificationBindings {
            repo,
            scope,
            rule,
            format,
        } => {
            let rule = rule.map(StableId::new).transpose()?;
            let bindings = StateStore::new(ProvenanceLayout::new(repo))
                .list_verification_bindings(&ScopeId::new(scope)?)?
                .into_iter()
                .filter(|binding| rule.as_ref().is_none_or(|rule| &binding.rule_id == rule))
                .collect::<Vec<_>>();
            output::print(format, &bindings)?;
        }
    }
    Ok(())
}

fn normalize_verification_context(
    repo: &camino::Utf8Path,
    input: &mut BeginVerificationInput,
) -> anyhow::Result<()> {
    let file = input
        .file
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("file is required for a durable verification binding"))?;
    let relative = if file.is_absolute() {
        file.strip_prefix(repo)
            .map_err(|_| {
                anyhow::anyhow!("verification file `{file}` is outside repository `{repo}`")
            })?
            .to_path_buf()
    } else {
        file.clone()
    };
    anyhow::ensure!(
        !relative
            .components()
            .any(|part| matches!(part, camino::Utf8Component::ParentDir)),
        "verification file must not leave the repository"
    );
    input.commit = clean_file_commit(repo, &relative);
    input.file = Some(relative);
    Ok(())
}

fn clean_file_commit(repo: &camino::Utf8Path, file: &camino::Utf8Path) -> Option<String> {
    let tracked = std::process::Command::new("git")
        .args(["ls-files", "--error-unmatch", "--", file.as_str()])
        .current_dir(repo)
        .output()
        .ok()?;
    if !tracked.status.success() {
        return None;
    }
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", file.as_str()])
        .current_dir(repo)
        .output()
        .ok()?;
    if !status.status.success() || !status.stdout.is_empty() {
        return None;
    }
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .ok()?;
    if !head.status.success() {
        return None;
    }
    Some(String::from_utf8(head.stdout).ok()?.trim().to_string())
}

fn read_stdin_json<T: serde::de::DeserializeOwned>() -> anyhow::Result<T> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    anyhow::ensure!(
        !input.trim().is_empty(),
        "expected a JSON document on stdin"
    );
    serde_json::from_str(&input).map_err(Into::into)
}
