use super::export::ScopeExport;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{validate_record_scope, validate_unique_ids, ScopeId};
use provenance_store::{
    layout::ProvenanceLayout, state_store::StateStore, transaction::StateTransaction,
};
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
    validate_export(&scope_id, &exported)?;
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
        + exported.promotion_decisions.len();
    if !dry_run {
        let layout = ProvenanceLayout::new(repo);
        let store = StateStore::new(layout.clone());
        store.write_transaction(|transaction| {
            import_records(transaction, &layout, &scope_id, &exported)
        })?;
    }
    Ok(ImportReport {
        status: "ok",
        dry_run,
        records,
    })
}

fn import_records(
    transaction: &mut StateTransaction,
    layout: &ProvenanceLayout,
    scope_id: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    transaction.replace_jsonl(
        &provenance_store::shards::sources_path(layout, scope_id),
        &exported.sources,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::domains_path(layout, scope_id),
        &exported.domains,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::requirements_path(layout, scope_id),
        &exported.requirements,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::boundaries_path(layout, scope_id),
        &exported.boundaries,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::topics_path(layout, scope_id),
        &exported.topics,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::questions_path(layout, scope_id),
        &exported.questions,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::resolutions_path(layout, scope_id),
        &exported.resolutions,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::rules_path(layout, scope_id),
        &exported.rules,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::services_path(layout, scope_id),
        &exported.services,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::service_bindings_path(layout, scope_id),
        &exported.service_bindings,
    )?;
    let edges_path = provenance_store::shards::edges_path(layout);
    let mut edges = transaction
        .read_jsonl::<provenance_core::Edge>(&edges_path)?
        .into_iter()
        .filter(|edge| edge.scope_id != *scope_id)
        .collect::<Vec<_>>();
    edges.extend(exported.edges.iter().cloned());
    edges.sort_by(|a, b| {
        a.scope_id
            .as_str()
            .cmp(b.scope_id.as_str())
            .then(a.id.as_str().cmp(b.id.as_str()))
    });
    transaction.replace_jsonl(&edges_path, &edges)?;
    transaction.replace_jsonl(
        &provenance_store::shards::threads_path(layout, scope_id),
        &exported.threads,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::messages_path(layout, scope_id),
        &exported.messages,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::contributions_path(layout, scope_id),
        &exported.contributions,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::synthesis_packets_path(layout, scope_id),
        &exported.synthesis_packets,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::proposal_cards_path(layout, scope_id),
        &exported.proposal_cards,
    )?;
    transaction.replace_jsonl(
        &provenance_store::shards::promotion_decisions_path(layout, scope_id),
        &exported.promotion_decisions,
    )?;
    Ok(())
}

fn validate_export(scope: &ScopeId, exported: &ScopeExport) -> anyhow::Result<()> {
    macro_rules! validate_scopes {
        ($kind:literal, $records:expr) => {
            for record in $records {
                validate_record_scope(scope, &record.scope_id, $kind, &record.id)?;
            }
        };
    }
    validate_scopes!("source", &exported.sources);
    validate_scopes!("domain", &exported.domains);
    validate_scopes!("requirement", &exported.requirements);
    validate_scopes!("boundary", &exported.boundaries);
    validate_scopes!("topic", &exported.topics);
    validate_scopes!("question", &exported.questions);
    validate_scopes!("resolution", &exported.resolutions);
    validate_scopes!("rule", &exported.rules);
    validate_scopes!("service", &exported.services);
    validate_scopes!("service binding", &exported.service_bindings);
    validate_scopes!("edge", &exported.edges);
    validate_scopes!("thread", &exported.threads);
    validate_scopes!("message", &exported.messages);
    validate_scopes!("contribution", &exported.contributions);
    validate_scopes!("synthesis packet", &exported.synthesis_packets);
    validate_scopes!("proposal", &exported.proposal_cards);
    validate_scopes!("promotion decision", &exported.promotion_decisions);
    validate_unique_ids("source", exported.sources.iter().map(|source| &source.id))?;
    for contribution in &exported.contributions {
        super::validate::validate_contribution_record(contribution)?;
    }
    for proposal in &exported.proposal_cards {
        super::validate::validate_proposal_card_record(proposal)?;
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
