use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) fn handle(repo: Utf8PathBuf, scope: String, format: OutputFormat) -> anyhow::Result<()> {
    let orphans = cache::orphan_rules(&ProvenanceLayout::new(repo), &ScopeId::new(scope)?)?;
    output::print(format, &orphans)?;
    Ok(())
}
