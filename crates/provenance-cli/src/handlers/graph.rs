use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{ScopeId, StableId};
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) fn handle(
    requirement_id: String,
    repo: Utf8PathBuf,
    scope: String,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let graph = cache::get_requirement_graph(
        &ProvenanceLayout::new(repo),
        &ScopeId::new(scope)?,
        &StableId::new(requirement_id)?,
    )?;
    output::print(format, &graph)?;
    Ok(())
}
