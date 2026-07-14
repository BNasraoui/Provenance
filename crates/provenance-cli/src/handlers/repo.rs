use camino::Utf8PathBuf;
use provenance_core::{Manifest, RepoPathPrefix, ScopeId};
use provenance_store::layout::ProvenanceLayout;

pub(super) fn init(
    path: Utf8PathBuf,
    scope: String,
    path_prefix: Utf8PathBuf,
    human_authority_ids: Vec<String>,
) -> anyhow::Result<()> {
    let layout = ProvenanceLayout::new(path);
    std::fs::create_dir_all(layout.scopes_dir())?;
    std::fs::create_dir_all(layout.edges_dir())?;
    std::fs::create_dir_all(layout.cache_dir())?;
    let mut manifest =
        Manifest::default_with_scope(ScopeId::new(scope)?, RepoPathPrefix::new(path_prefix));
    manifest.human_authority_ids = human_authority_ids;
    std::fs::write(
        layout.manifest_path(),
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )?;
    Ok(())
}
