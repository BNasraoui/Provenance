use super::export::ScopeExport;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{validate_ideation_aggregate, IdeationAggregate, ScopeId};
use provenance_store::layout::ProvenanceLayout;
use provenance_store::state_store::{IdeationLandingBatch, StateStore};
use serde::Serialize;

#[derive(Serialize)]
pub struct ImportReport {
    pub status: &'static str,
    pub dry_run: bool,
    pub records: usize,
}

#[allow(clippy::too_many_lines)]
pub(super) fn import_scope(
    repo: Utf8PathBuf,
    scope: &str,
    input: Utf8PathBuf,
    dry_run: bool,
) -> anyhow::Result<ImportReport> {
    let exported: ScopeExport = serde_json::from_str(&std::fs::read_to_string(input)?)?;
    anyhow::ensure!(
        exported.scope == scope,
        "import scope does not match --scope"
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
        + exported.promotion_decisions.len();
    let scope_id = ScopeId::new(scope)?;
    for (kind, actual) in exported
        .contributions
        .iter()
        .map(|record| ("contribution", &record.scope_id))
        .chain(
            exported
                .synthesis_packets
                .iter()
                .map(|record| ("synthesis packet", &record.scope_id)),
        )
        .chain(
            exported
                .proposal_cards
                .iter()
                .map(|record| ("proposal", &record.scope_id)),
        )
        .chain(
            exported
                .assertion_records
                .iter()
                .map(|record| ("assertion", &record.scope_id)),
        )
        .chain(
            exported
                .promotion_decisions
                .iter()
                .map(|record| ("disposition", &record.scope_id)),
        )
    {
        anyhow::ensure!(
            actual == &scope_id,
            "{kind} scope_id must match imported scope"
        );
    }
    validate_ideation_aggregate(IdeationAggregate {
        contributions: &exported.contributions,
        synthesis_packets: &exported.synthesis_packets,
        proposals: &exported.proposal_cards,
        assertions: &exported.assertion_records,
        dispositions: &exported.promotion_decisions,
    })?;
    if !dry_run {
        let layout = ProvenanceLayout::new(repo);
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::sources_path(&layout, &scope_id),
            &exported.sources,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::domains_path(&layout, &scope_id),
            &exported.domains,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::requirements_path(&layout, &scope_id),
            &exported.requirements,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::boundaries_path(&layout, &scope_id),
            &exported.boundaries,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::topics_path(&layout, &scope_id),
            &exported.topics,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::questions_path(&layout, &scope_id),
            &exported.questions,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::resolutions_path(&layout, &scope_id),
            &exported.resolutions,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::rules_path(&layout, &scope_id),
            &exported.rules,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::services_path(&layout, &scope_id),
            &exported.services,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::service_bindings_path(&layout, &scope_id),
            &exported.service_bindings,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::edges_path(&layout),
            &exported.edges,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::threads_path(&layout, &scope_id),
            &exported.threads,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::messages_path(&layout, &scope_id),
            &exported.messages,
        )?;
        StateStore::new(layout).land_ideation_batch(
            &scope_id,
            IdeationLandingBatch {
                contributions: exported.contributions,
                synthesis_packets: exported.synthesis_packets,
                proposals: exported.proposal_cards,
                assertions: exported.assertion_records,
                dispositions: exported.promotion_decisions,
            },
            false,
        )?;
    }
    Ok(ImportReport {
        status: "ok",
        dry_run,
        records,
    })
}

pub(super) fn handle(
    repo: Utf8PathBuf,
    scope: &str,
    input: Utf8PathBuf,
    dry_run: bool,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let report = import_scope(repo, scope, input, dry_run)?;
    output::print(format, &report)?;
    Ok(())
}
