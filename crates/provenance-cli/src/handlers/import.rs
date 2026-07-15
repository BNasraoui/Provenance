use super::export::ScopeExport;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{ScopeReplacement, StateStore},
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
    validate_export_semantics(&exported)?;
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
    let replacement = ScopeReplacement::from(exported);
    replacement.validate_for_scope(&scope_id)?;
    let store = StateStore::new(ProvenanceLayout::new(repo));
    if dry_run {
        anyhow::ensure!(
            store
                .manifest()?
                .scopes
                .iter()
                .any(|manifest_scope| manifest_scope.id == scope_id),
            "scope {} is absent from manifest",
            scope_id.as_str()
        );
    } else {
        store.replace_scope(&scope_id, &replacement)?;
    }
    Ok(ImportReport {
        status: "ok",
        dry_run,
        records,
    })
}

fn validate_export_semantics(exported: &ScopeExport) -> anyhow::Result<()> {
    for contribution in &exported.contributions {
        super::validate::validate_contribution_record(contribution)?;
    }
    for proposal in &exported.proposal_cards {
        super::validate::validate_proposal_card_record(proposal)?;
    }
    Ok(())
}

impl From<ScopeExport> for ScopeReplacement {
    fn from(exported: ScopeExport) -> Self {
        Self {
            sources: exported.sources,
            domains: exported.domains,
            requirements: exported.requirements,
            boundaries: exported.boundaries,
            topics: exported.topics,
            questions: exported.questions,
            resolutions: exported.resolutions,
            rules: exported.rules,
            services: exported.services,
            service_bindings: exported.service_bindings,
            edges: exported.edges,
            threads: exported.threads,
            messages: exported.messages,
            contributions: exported.contributions,
            synthesis_packets: exported.synthesis_packets,
            proposal_cards: exported.proposal_cards,
            promotion_decisions: exported.promotion_decisions,
        }
    }
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
