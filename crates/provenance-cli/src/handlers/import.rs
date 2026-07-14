use super::export::ScopeExport;
use super::{check, export};
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{
    validate_ideation_aggregate, validate_proposal_intrinsic, IdeationAggregate, ProposalType,
    ScopeId,
};
use provenance_store::layout::ProvenanceLayout;
use provenance_store::state_store::{IdeationLandingBatch, StateStore};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
pub struct ImportReport {
    pub status: &'static str,
    pub dry_run: bool,
    pub records: usize,
}

#[allow(clippy::too_many_lines)]
pub(super) fn import_scope(
    repo: &camino::Utf8Path,
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
    validate_scope_ownership(&exported, &scope_id)?;
    for proposal in &exported.proposal_cards {
        validate_proposal_intrinsic(proposal)?;
    }
    let layout = ProvenanceLayout::new(repo.to_path_buf());
    let store = StateStore::new(layout);
    store.with_state_transaction(|| {
        preflight_ideation_import(&store, &scope_id, &exported)?;
        reject_unauthorized_imported_dispositions(&store, &scope_id, &exported)?;
        if dry_run {
            validate_staged_import(repo, &scope_id, &exported)
        } else {
            commit_staged_import(repo, &scope_id, &exported)
        }
    })?;
    Ok(ImportReport {
        status: "ok",
        dry_run,
        records,
    })
}

fn apply_import(
    repo: &camino::Utf8Path,
    store: &StateStore,
    scope_id: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    let layout = ProvenanceLayout::new(repo.to_path_buf());
    store.land_ideation_batch(
        scope_id,
        IdeationLandingBatch {
            contributions: exported.contributions.clone(),
            synthesis_packets: exported.synthesis_packets.clone(),
            proposals: exported.proposal_cards.clone(),
            assertions: exported.assertion_records.clone(),
            dispositions: exported.promotion_decisions.clone(),
        },
        false,
    )?;
    let existing = export::export_scope(repo.to_path_buf(), scope_id.as_str().to_owned())?;
    let merged = merge_scope_exports(existing, exported)?;
    let edge_path = provenance_store::shards::edges_path(&layout);
    let mut direct_edges = read_jsonl_file(&edge_path)?;
    direct_edges.extend(exported.edges.iter().cloned());
    let message_path = provenance_store::shards::messages_path(&layout, scope_id);
    let mut direct_messages = read_jsonl_file(&message_path)?;
    direct_messages.extend(exported.messages.iter().cloned());
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::sources_path(&layout, scope_id),
        &merged.sources,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::domains_path(&layout, scope_id),
        &merged.domains,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::requirements_path(&layout, scope_id),
        &merged.requirements,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::boundaries_path(&layout, scope_id),
        &merged.boundaries,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::topics_path(&layout, scope_id),
        &merged.topics,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::questions_path(&layout, scope_id),
        &merged.questions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::resolutions_path(&layout, scope_id),
        &merged.resolutions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::rules_path(&layout, scope_id),
        &merged.rules,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::services_path(&layout, scope_id),
        &merged.services,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::service_bindings_path(&layout, scope_id),
        &merged.service_bindings,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(&edge_path, &direct_edges)?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::threads_path(&layout, scope_id),
        &merged.threads,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(&message_path, &direct_messages)?;
    Ok(())
}

fn validate_scope_ownership(exported: &ScopeExport, expected: &ScopeId) -> anyhow::Result<()> {
    macro_rules! owned {
        ($kind:literal, $records:expr) => {
            for record in $records {
                anyhow::ensure!(
                    &record.scope_id == expected,
                    "{} scope_id must match imported scope",
                    $kind
                );
            }
        };
    }
    owned!("source", &exported.sources);
    owned!("domain", &exported.domains);
    owned!("requirement", &exported.requirements);
    owned!("boundary", &exported.boundaries);
    owned!("topic", &exported.topics);
    owned!("question", &exported.questions);
    owned!("resolution", &exported.resolutions);
    owned!("rule", &exported.rules);
    owned!("service", &exported.services);
    owned!("service binding", &exported.service_bindings);
    owned!("edge", &exported.edges);
    owned!("thread", &exported.threads);
    owned!("message", &exported.messages);
    owned!("contribution", &exported.contributions);
    owned!("synthesis packet", &exported.synthesis_packets);
    owned!("proposal", &exported.proposal_cards);
    owned!("assertion", &exported.assertion_records);
    owned!("disposition", &exported.promotion_decisions);
    Ok(())
}

fn reject_unauthorized_imported_dispositions(
    store: &StateStore,
    scope: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    let manifest = store.manifest()?;
    let mut proposals = store.list_proposal_definitions(scope)?;
    proposals.extend(exported.proposal_cards.iter().cloned());
    for disposition in &exported.promotion_decisions {
        let proposal = proposals
            .iter()
            .find(|proposal| proposal.id == disposition.proposal_id)
            .ok_or_else(|| anyhow::anyhow!("disposition proposal does not exist"))?;
        if matches!(
            proposal.proposal_type,
            ProposalType::RequirementCandidate
                | ProposalType::ResolutionCandidate
                | ProposalType::RuleCandidate
        ) {
            anyhow::ensure!(
                disposition.actor.identity_type == provenance_core::IdentityType::Human
                    && manifest
                        .human_authority_ids
                        .iter()
                        .any(|trusted| trusted == &disposition.actor.id),
                "behavior-changing disposition actor is not a repository-configured human authority"
            );
        }
    }
    Ok(())
}

fn merge_scope_exports(
    mut existing: ScopeExport,
    incoming: &ScopeExport,
) -> anyhow::Result<ScopeExport> {
    macro_rules! append {
        ($field:ident, $kind:literal) => {
            append_disjoint($kind, &mut existing.$field, &incoming.$field)?;
        };
    }
    append!(sources, "source");
    append!(domains, "domain");
    append!(requirements, "requirement");
    append!(boundaries, "boundary");
    append!(topics, "topic");
    append!(questions, "question");
    append!(resolutions, "resolution");
    append!(rules, "rule");
    append!(services, "service");
    append!(service_bindings, "service binding");
    append!(edges, "edge");
    append!(threads, "thread");
    append!(messages, "message");
    Ok(existing)
}

fn append_disjoint<T: Clone + Serialize>(
    kind: &str,
    existing: &mut Vec<T>,
    incoming: &[T],
) -> anyhow::Result<()> {
    let mut ids = existing
        .iter()
        .map(record_id)
        .collect::<anyhow::Result<BTreeSet<_>>>()?;
    for record in incoming {
        let id = record_id(record)?;
        anyhow::ensure!(ids.insert(id.clone()), "{kind} {id} already exists");
        existing.push(record.clone());
    }
    Ok(())
}

fn record_id<T: Serialize>(record: &T) -> anyhow::Result<String> {
    serde_json::to_value(record)?
        .get("id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("imported record has no string id"))
}

fn read_jsonl_file<T: serde::de::DeserializeOwned>(
    path: &camino::Utf8Path,
) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

fn validate_staged_import(
    repo: &camino::Utf8Path,
    scope: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    let (transaction, staged_repo, _) = prepare_staged_repo(repo, scope)?;
    let staged_store = StateStore::new(ProvenanceLayout::new(staged_repo.clone()));
    let result = apply_import(&staged_repo, &staged_store, scope, exported)
        .and_then(|()| check::validate_repository(staged_repo));
    let _ = std::fs::remove_dir_all(transaction);
    result
}

fn commit_staged_import(
    repo: &camino::Utf8Path,
    scope: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    let (transaction, staged_repo, backup) = prepare_staged_repo(repo, scope)?;
    let staged_store = StateStore::new(ProvenanceLayout::new(staged_repo.clone()));
    apply_import(&staged_repo, &staged_store, scope, exported)?;
    check::validate_repository(staged_repo.clone())?;

    let live_state = ProvenanceLayout::new(repo.to_path_buf()).state_dir();
    let staged_state = ProvenanceLayout::new(staged_repo).state_dir();
    std::fs::rename(&live_state, &backup)?;
    if let Err(error) = std::fs::rename(&staged_state, &live_state) {
        std::fs::rename(&backup, &live_state).map_err(|rollback| {
            anyhow::anyhow!("state publish failed: {error}; rollback failed: {rollback}")
        })?;
        return Err(error.into());
    }
    // Publication is committed once staged state is live. Cleanup is
    // recoverable on the next import and must not turn success into a retry
    // that collides with already-committed identities.
    let _ = std::fs::remove_dir_all(&backup);
    let _ = std::fs::remove_dir_all(&transaction);
    Ok(())
}

fn prepare_staged_repo(
    repo: &camino::Utf8Path,
    scope: &ScopeId,
) -> anyhow::Result<(Utf8PathBuf, Utf8PathBuf, Utf8PathBuf)> {
    let layout = ProvenanceLayout::new(repo.to_path_buf());
    let transaction = layout
        .cache_dir()
        .join("import-transactions")
        .join(scope.as_str());
    let staged_repo = transaction.join("staged-repo");
    let backup = transaction.join("backup-state");
    let live_state = layout.state_dir();

    std::fs::create_dir_all(&transaction)?;
    if backup.exists() {
        if live_state.exists() {
            std::fs::remove_dir_all(&backup)?;
        } else {
            std::fs::rename(&backup, &live_state)?;
        }
    }
    if staged_repo.exists() {
        std::fs::remove_dir_all(&staged_repo)?;
    }
    let staged_state = ProvenanceLayout::new(staged_repo.clone()).state_dir();
    copy_directory(&live_state, &staged_state)?;
    Ok((transaction, staged_repo, backup))
}

fn copy_directory(source: &camino::Utf8Path, destination: &camino::Utf8Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        let target = destination.join(name.to_string_lossy().as_ref());
        let source = camino::Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| anyhow::anyhow!("state path is not UTF-8"))?;
        if entry.file_type()?.is_dir() {
            copy_directory(&source, &target)?;
        } else {
            std::fs::copy(source, target)?;
        }
    }
    Ok(())
}

fn preflight_ideation_import(
    store: &StateStore,
    scope_id: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    let existing_contributions = store.list_contributions(scope_id)?;
    let existing_synthesis = store.list_synthesis_packets(scope_id)?;
    let existing_proposals = store.list_proposal_definitions(scope_id)?;
    let existing_assertions = store.list_assertion_records(scope_id)?;
    let existing_dispositions = store.list_promotion_decisions(scope_id)?;

    ensure_disjoint(
        "contribution",
        existing_contributions
            .iter()
            .map(|record| record.id.as_str()),
        exported
            .contributions
            .iter()
            .map(|record| record.id.as_str()),
    )?;
    ensure_disjoint(
        "synthesis packet",
        existing_synthesis.iter().map(|record| record.id.as_str()),
        exported
            .synthesis_packets
            .iter()
            .map(|record| record.id.as_str()),
    )?;
    ensure_disjoint(
        "proposal",
        existing_proposals.iter().map(|record| record.id.as_str()),
        exported
            .proposal_cards
            .iter()
            .map(|record| record.id.as_str()),
    )?;
    ensure_disjoint(
        "assertion",
        existing_assertions.iter().map(|record| record.id.as_str()),
        exported
            .assertion_records
            .iter()
            .map(|record| record.id.as_str()),
    )?;
    ensure_disjoint(
        "disposition",
        existing_dispositions
            .iter()
            .map(|record| record.id.as_str()),
        exported
            .promotion_decisions
            .iter()
            .map(|record| record.id.as_str()),
    )?;

    let mut contributions = existing_contributions;
    contributions.extend(exported.contributions.iter().cloned());
    let mut synthesis_packets = existing_synthesis;
    synthesis_packets.extend(exported.synthesis_packets.iter().cloned());
    let mut proposals = existing_proposals;
    proposals.extend(exported.proposal_cards.iter().cloned());
    let mut assertions = existing_assertions;
    assertions.extend(exported.assertion_records.iter().cloned());
    let mut dispositions = existing_dispositions;
    dispositions.extend(exported.promotion_decisions.iter().cloned());
    validate_ideation_aggregate(IdeationAggregate {
        contributions: &contributions,
        synthesis_packets: &synthesis_packets,
        proposals: &proposals,
        assertions: &assertions,
        dispositions: &dispositions,
    })
}

fn ensure_disjoint<'a>(
    kind: &str,
    existing: impl IntoIterator<Item = &'a str>,
    incoming: impl IntoIterator<Item = &'a str>,
) -> anyhow::Result<()> {
    let existing = existing.into_iter().collect::<BTreeSet<_>>();
    for id in incoming {
        anyhow::ensure!(!existing.contains(id), "{kind} {id} already exists");
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn handle(
    repo: Utf8PathBuf,
    scope: &str,
    input: Utf8PathBuf,
    dry_run: bool,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let report = import_scope(&repo, scope, input, dry_run)?;
    output::print(format, &report)?;
    Ok(())
}
