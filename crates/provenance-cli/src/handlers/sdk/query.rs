use crate::cli::sdk::QueryArgs;
use crate::output;
use provenance_core::protocol::QueryResponse;
use provenance_core::ScopeId;
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};

mod bindings;
mod evidence;
mod impact;
mod records;
mod stale;
mod symbols;
mod walk;

/// Which structured query the caller asked for.
#[derive(Debug, Clone, Copy)]
pub(super) enum Operation {
    Get,
    Search,
    Neighbors,
    Trace,
    Impact,
    Evidence,
    Stale,
    ResolveSymbol,
}

/// Runs one structured query and writes its bounded answer.
///
/// Every primitive is one named operation with typed parameters read from
/// stdin, and every answer carries the protocol version that produced it.
pub(super) fn handle(operation: Operation, args: QueryArgs) -> anyhow::Result<()> {
    let repo = super::resolve_repository(args.repo)?;
    let scope = ScopeId::new(args.scope)?;
    let store = StateStore::new(ProvenanceLayout::new(repo.clone()));
    let format = args.format;
    match operation {
        Operation::Get => {
            let result = records::get(&store, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("get", result))
        }
        Operation::Search => {
            let result = records::search(&store, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("search", result))
        }
        Operation::Neighbors => {
            let result = walk::neighbors(&store, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("neighbors", result))
        }
        Operation::Trace => {
            let result = walk::trace(&store, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("trace", result))
        }
        Operation::Impact => {
            let result = impact::impact(&repo, &store, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("impact", result))
        }
        Operation::Evidence => {
            let result = evidence::evidence(&repo, &store, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("evidence", result))
        }
        Operation::Stale => {
            let result = stale::stale(&repo, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("stale", result))
        }
        Operation::ResolveSymbol => {
            let result = symbols::resolve(&repo, &store, &scope, super::read_stdin_json()?)?;
            output::print(format, &QueryResponse::new("resolve-symbol", result))
        }
    }
}
