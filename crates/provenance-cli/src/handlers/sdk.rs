use std::io::Read as _;
use std::str::FromStr as _;

use crate::cli::sdk::SdkCommand;
use crate::output;
use provenance_core::{
    EngineInfo, ScopeId, StableId, SDK_PROTOCOL_VERSION, SUPPORTED_SCHEMA_VERSION,
};
use provenance_macros::rule;
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{BeginVerificationInput, CompleteVerificationInput, StateStore, TypedSpecInput},
};

mod plan;

pub(super) fn handle(command: SdkCommand) -> anyhow::Result<()> {
    match command {
        SdkCommand::Info { repo, format } => {
            let repo = resolve_repository(repo)?;
            output::print(
                format,
                &EngineInfo {
                    engine_version: env!("CARGO_PKG_VERSION").to_string(),
                    protocol_version: SDK_PROTOCOL_VERSION,
                    state_schema_version: SUPPORTED_SCHEMA_VERSION.0,
                    repository: repo,
                },
            )?;
        }
        SdkCommand::Plan {
            repo,
            scope,
            format,
        } => {
            let repo = resolve_repository(repo)?;
            let mut input = read_stdin_json::<TypedSpecInput>()?;
            normalize_implementation_context(&repo, &mut input)?;
            let result = plan::typed_spec(&repo, &ScopeId::new(scope)?, input)?;
            output::print(format, &result)?;
        }
        SdkCommand::Apply {
            repo,
            scope,
            format,
        } => {
            let repo = resolve_repository(repo)?;
            let mut input = read_stdin_json::<TypedSpecInput>()?;
            normalize_implementation_context(&repo, &mut input)?;
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
            let repo = resolve_repository(repo)?;
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
            let repo = resolve_repository(repo)?;
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
            let repo = resolve_repository(repo)?;
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
            let repo = resolve_repository(repo)?;
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

/// Resolves one explicit root or discovers the nearest enclosing project.
#[rule("rule_sdk_project_discovery")]
fn resolve_repository(repo: Option<camino::Utf8PathBuf>) -> anyhow::Result<camino::Utf8PathBuf> {
    let cwd = camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|path| anyhow::anyhow!("current directory is not UTF-8: {}", path.display()))?;
    let (start, explicit) = match repo {
        Some(repo) => (
            if repo.is_relative() {
                cwd.join(repo)
            } else {
                repo
            },
            true,
        ),
        None => (cwd, false),
    };
    if explicit {
        return canonical_repository(&start);
    }
    for ancestor in start.ancestors() {
        if ancestor.join(".provenance/state/manifest.json").is_file()
            || ancestor.join(".git").exists()
        {
            return canonical_repository(ancestor);
        }
    }
    canonical_repository(&start)
}

fn canonical_repository(path: &camino::Utf8Path) -> anyhow::Result<camino::Utf8PathBuf> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|error| anyhow::anyhow!("resolve repository `{path}`: {error}"))?;
    camino::Utf8PathBuf::from_path_buf(canonical)
        .map_err(|path| anyhow::anyhow!("repository is not UTF-8: {}", path.display()))
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
        let canonical = std::fs::canonicalize(file).map_err(|error| {
            anyhow::anyhow!("verification file `{file}` cannot be resolved: {error}")
        })?;
        let canonical = camino::Utf8PathBuf::from_path_buf(canonical).map_err(|path| {
            anyhow::anyhow!("verification file is not UTF-8: {}", path.display())
        })?;
        canonical
            .strip_prefix(repo)
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
    let relative = portable_repository_path(&relative)?;
    input.commit = clean_file_commit(repo, &relative);
    input.file = Some(relative);
    Ok(())
}

fn normalize_implementation_context(
    repo: &camino::Utf8Path,
    input: &mut TypedSpecInput,
) -> anyhow::Result<()> {
    for rule in &mut input.rules {
        let Some(implementation) = &mut rule.implementation else {
            continue;
        };
        let candidate = if implementation.file.is_absolute() {
            implementation.file.clone()
        } else {
            repo.join(&implementation.file)
        };
        let canonical = std::fs::canonicalize(&candidate).map_err(|error| {
            anyhow::anyhow!(
                "implementation target `{}` does not exist or cannot be resolved: {error}",
                implementation.file
            )
        })?;
        let canonical = camino::Utf8PathBuf::from_path_buf(canonical).map_err(|path| {
            anyhow::anyhow!("implementation target is not UTF-8: {}", path.display())
        })?;
        anyhow::ensure!(
            canonical.is_file(),
            "implementation target `{canonical}` is not a file"
        );
        let relative = canonical
            .strip_prefix(repo)
            .map_err(|_| {
                anyhow::anyhow!(
                    "implementation target `{canonical}` is outside repository `{repo}`"
                )
            })?
            .to_path_buf();
        implementation.file = portable_repository_path(&relative)?;
    }
    Ok(())
}

fn portable_repository_path(path: &camino::Utf8Path) -> anyhow::Result<camino::Utf8PathBuf> {
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            camino::Utf8Component::Normal(segment) => segments.push(segment),
            camino::Utf8Component::CurDir => {}
            camino::Utf8Component::ParentDir
            | camino::Utf8Component::RootDir
            | camino::Utf8Component::Prefix(_) => {
                anyhow::bail!("path must be repository-relative")
            }
        }
    }
    anyhow::ensure!(!segments.is_empty(), "path must name a repository file");
    Ok(segments.join("/").into())
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
