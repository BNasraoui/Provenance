use super::export::ScopeExport;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{Edge, ScopeId};
use provenance_store::layout::ProvenanceLayout;
use provenance_store::state_store::StateStore;
use serde::Serialize;

#[derive(Serialize)]
pub struct ImportReport {
    pub status: &'static str,
    pub dry_run: bool,
    pub records: usize,
}

pub(super) fn import_scope(
    repo: Utf8PathBuf,
    scope: String,
    input: Utf8PathBuf,
    dry_run: bool,
) -> anyhow::Result<ImportReport> {
    let exported: ScopeExport = serde_json::from_str(&std::fs::read_to_string(input)?)?;
    anyhow::ensure!(
        exported.scope == scope,
        "import scope does not match --scope"
    );
    let scope_id = ScopeId::new(scope)?;
    anyhow::ensure!(
        exported.edges.iter().all(|edge| edge.scope_id == scope_id),
        "edge scope_id must match import scope"
    );
    let records = exported.sources.len()
        + exported.domains.len()
        + exported.requirements.len()
        + exported.boundaries.len()
        + exported.topics.len()
        + exported.questions.len()
        + exported.resolutions.len()
        + exported.rules.len()
        + exported.services.len()
        + exported.service_bindings.len()
        + exported.edges.len()
        + exported.threads.len()
        + exported.messages.len()
        + exported.contributions.len()
        + exported.synthesis_packets.len()
        + exported.proposal_cards.len()
        + exported.assertion_records.len()
        + exported.dispositions.len();
    let live_layout = ProvenanceLayout::new(repo);
    let manifest = StateStore::new(live_layout.clone()).manifest()?;
    provenance_core::validate_ideation_aggregate(provenance_core::IdeationAggregate {
        legacy_policy: provenance_core::LegacyProposalPolicy::ShippedV1,
        disposition_actor_ids: &manifest.disposition_actor_ids,
        contributions: &exported.contributions,
        synthesis_packets: &exported.synthesis_packets,
        proposals: &exported.proposal_cards,
        assertions: &exported.assertion_records,
        dispositions: &exported.dispositions,
    })?;
    provenance_store::publication::with_repository_publication(&live_layout, || {
        apply_import(&live_layout, &scope_id, &exported, dry_run)
    })?;
    Ok(ImportReport {
        status: "ok",
        dry_run,
        records,
    })
}

fn apply_import(
    live_layout: &ProvenanceLayout,
    scope_id: &ScopeId,
    exported: &ScopeExport,
    dry_run: bool,
) -> anyhow::Result<()> {
    let transaction = live_layout.import_transactions_dir().join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let staged_repo = transaction.join("staged-repo");
    copy_directory(
        &live_layout.state_dir(),
        &ProvenanceLayout::new(staged_repo.clone()).state_dir(),
    )?;
    let layout = ProvenanceLayout::new(staged_repo.clone());
    let staged_scope = layout.scopes_dir().join(scope_id.as_str());
    if staged_scope.exists() {
        std::fs::remove_dir_all(staged_scope)?;
    }
    write_scope(&layout, scope_id, exported)?;
    super::check::validate_repository(staged_repo)?;
    if !dry_run {
        provenance_store::publication::sync_tree(&layout.state_dir())?;
        let backup = transaction.join("backup-state");
        provenance_store::publication::write_publication_marker(
            live_layout,
            &transaction,
            provenance_store::publication::PublicationPhase::Prepared,
        )?;
        std::fs::rename(live_layout.state_dir(), &backup)?;
        if let Err(error) =
            provenance_store::publication::sync_directory(&live_layout.provenance_dir())
                .and_then(|()| {
                    provenance_store::publication::write_publication_marker(
                        live_layout,
                        &transaction,
                        provenance_store::publication::PublicationPhase::BackupCreated,
                    )
                })
                .and_then(|()| {
                    std::fs::rename(layout.state_dir(), live_layout.state_dir()).map_err(Into::into)
                })
                .and_then(|()| {
                    provenance_store::publication::sync_directory(&live_layout.provenance_dir())
                })
                .and_then(|()| {
                    provenance_store::publication::write_publication_marker(
                        live_layout,
                        &transaction,
                        provenance_store::publication::PublicationPhase::Published,
                    )
                })
        {
            rollback_publication(live_layout, &layout, &backup)?;
            return Err(error);
        }
        if std::fs::remove_dir_all(&transaction).is_ok() {
            let _ = provenance_store::publication::clear_publication_marker(live_layout);
        }
        return Ok(());
    }
    std::fs::remove_dir_all(transaction)?;
    Ok(())
}

fn rollback_publication(
    live_layout: &ProvenanceLayout,
    staged_layout: &ProvenanceLayout,
    backup: &camino::Utf8Path,
) -> anyhow::Result<()> {
    if live_layout.state_dir().exists() {
        std::fs::rename(live_layout.state_dir(), staged_layout.state_dir())?;
    }
    if backup.exists() {
        std::fs::rename(backup, live_layout.state_dir())?;
    }
    provenance_store::publication::sync_directory(&live_layout.provenance_dir())?;
    provenance_store::publication::clear_publication_marker(live_layout)
}

fn write_scope(
    layout: &ProvenanceLayout,
    scope_id: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::sources_path(layout, scope_id),
        &exported.sources,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::domains_path(layout, scope_id),
        &exported.domains,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::requirements_path(layout, scope_id),
        &exported.requirements,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::boundaries_path(layout, scope_id),
        &exported.boundaries,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::topics_path(layout, scope_id),
        &exported.topics,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::questions_path(layout, scope_id),
        &exported.questions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::resolutions_path(layout, scope_id),
        &exported.resolutions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::rules_path(layout, scope_id),
        &exported.rules,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::services_path(layout, scope_id),
        &exported.services,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::service_bindings_path(layout, scope_id),
        &exported.service_bindings,
    )?;
    let edge_path = provenance_store::shards::edges_path(layout);
    let mut edges: Vec<Edge> = StateStore::new(layout.clone()).list_edges()?;
    edges.retain(|edge| edge.scope_id != *scope_id);
    edges.extend(exported.edges.iter().cloned());
    edges.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    remove_edge_shards(layout)?;
    provenance_store::jsonl::write_jsonl_atomic(&edge_path, &edges)?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::threads_path(layout, scope_id),
        &exported.threads,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::messages_path(layout, scope_id),
        &exported.messages,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::contributions_path(layout, scope_id),
        &exported.contributions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::synthesis_packets_path(layout, scope_id),
        &exported.synthesis_packets,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::proposal_cards_path(layout, scope_id),
        &exported.proposal_cards,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::assertion_records_path(layout, scope_id),
        &exported.assertion_records,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::dispositions_path(layout, scope_id),
        &exported.dispositions,
    )?;
    Ok(())
}

fn remove_edge_shards(layout: &ProvenanceLayout) -> anyhow::Result<()> {
    if !layout.edges_dir().exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(layout.edges_dir())? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some("jsonl")
        {
            std::fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn copy_directory(source: &camino::Utf8Path, destination: &camino::Utf8Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|path| anyhow::anyhow!("state path is not UTF-8: {}", path.display()))?;
        let target = destination.join(entry.file_name().to_string_lossy().as_ref());
        if entry.file_type()?.is_dir() {
            copy_directory(&source_path, &target)?;
        } else {
            std::fs::copy(source_path, target)?;
        }
    }
    Ok(())
}

pub(super) fn handle(
    repo: Utf8PathBuf,
    scope: String,
    input: Utf8PathBuf,
    dry_run: bool,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let report = import_scope(repo, scope, input, dry_run)?;
    output::print(format, &report)?;
    Ok(())
}
