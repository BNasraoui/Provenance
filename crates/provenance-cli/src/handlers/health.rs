use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) fn handle(repo: Utf8PathBuf, scope: String, format: OutputFormat) -> anyhow::Result<()> {
    let health = cache::coverage_health(&ProvenanceLayout::new(repo), &ScopeId::new(scope)?)?;
    output::print(format, &health)?;
    Ok(())
}
