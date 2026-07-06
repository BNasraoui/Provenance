use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) fn handle(repo: Utf8PathBuf, scope: String, format: OutputFormat) -> anyhow::Result<()> {
    let gaps = cache::find_gaps(&ProvenanceLayout::new(repo), &ScopeId::new(scope)?)?;
    output::print(format, &gaps)?;
    Ok(())
}
