use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) async fn handle(repo: Utf8PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let report = cache::materialize_state(&ProvenanceLayout::new(repo)).await?;
    output::print(format, &report)?;
    Ok(())
}
