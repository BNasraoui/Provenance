use crate::cli::Status;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{Manifest, RepoPathPrefix, ScopeId};
use provenance_store::layout::ProvenanceLayout;

pub(super) fn init(
    path: Utf8PathBuf,
    scope: String,
    path_prefix: Utf8PathBuf,
) -> anyhow::Result<()> {
    let layout = ProvenanceLayout::new(path);
    std::fs::create_dir_all(layout.scopes_dir())?;
    std::fs::create_dir_all(layout.edges_dir())?;
    std::fs::create_dir_all(layout.cache_dir())?;
    let manifest =
        Manifest::default_with_scope(ScopeId::new(scope)?, RepoPathPrefix::new(path_prefix));
    std::fs::write(
        layout.manifest_path(),
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )?;
    Ok(())
}

pub(super) fn check(repo: Utf8PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let manifest_path = ProvenanceLayout::new(repo).manifest_path();
    let manifest: Manifest = serde_json::from_str(&std::fs::read_to_string(manifest_path)?)?;
    anyhow::ensure!(
        !manifest.scopes.is_empty(),
        "manifest must contain at least one scope"
    );
    output::print(format, &Status { status: "ok" })
}
