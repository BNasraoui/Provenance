use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{ScopeId, StableId};
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) fn handle(
    rule_id: String,
    repo: Utf8PathBuf,
    scope: String,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let trace = cache::trace_rule(
        &ProvenanceLayout::new(repo),
        &ScopeId::new(scope)?,
        &StableId::new(rule_id)?,
    )?;
    output::print(format, &trace)?;
    Ok(())
}
