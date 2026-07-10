use crate::cli::Status;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};
use std::collections::BTreeSet;

mod edges;
mod index;
mod references;
mod scope;

use index::CheckIndex;

pub(super) fn check(repo: Utf8PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let store = StateStore::new(ProvenanceLayout::new(repo));
    let manifest = store.manifest()?;
    anyhow::ensure!(
        !manifest.scopes.is_empty(),
        "manifest must contain at least one scope"
    );
    let manifest_scopes: BTreeSet<_> = manifest
        .scopes
        .iter()
        .map(|scope| scope.id.as_str().to_string())
        .collect();

    let scope_directory_findings = store
        .list_scope_directories()?
        .into_iter()
        .filter(|directory| !manifest_scopes.contains(directory))
        .map(|directory| format!("scope directory {directory} is absent from manifest"))
        .collect::<Vec<_>>();

    let mut index = CheckIndex::default();
    let mut dangling = Vec::new();
    for scope in &manifest.scopes {
        let records = scope::ScopeRecords::load(&store, &scope.id)?;
        records.add_to(&mut index);
        scope::core::validate_sources_and_requirements(&records, &index, &scope.id, &mut dangling);
        scope::core::validate_shaping(&records, &index, &scope.id, &mut dangling);
        scope::core::validate_decisions(&records, &index, &scope.id, &mut dangling);
        scope::collaboration::validate(&records, &index, &scope.id, &mut dangling);
        scope::ideation::validate(&records, &index, &scope.id, &mut dangling);
    }
    edges::validate(&store, &manifest_scopes, &index, &mut dangling)?;

    anyhow::ensure!(
        scope_directory_findings.is_empty(),
        "scope directory finding(s):\n- {}",
        scope_directory_findings.join("\n- ")
    );
    anyhow::ensure!(
        dangling.is_empty(),
        "dangling reference(s):\n- {}",
        dangling.join("\n- ")
    );
    output::print(format, &Status { status: "ok" })
}
