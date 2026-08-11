use std::io::Read as _;
use std::str::FromStr as _;

use crate::cli::sdk::SdkCommand;
use crate::output;
use provenance_core::{ScopeId, StableId};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{BeginVerificationInput, CompleteVerificationInput, StateStore, TypedSpecInput},
};

pub(super) fn handle(command: SdkCommand) -> anyhow::Result<()> {
    match command {
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
    }
    Ok(())
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
