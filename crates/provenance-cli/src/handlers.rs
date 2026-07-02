use crate::cli::Status;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{Manifest, RepoPathPrefix, ScopeId};
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt::Write;

#[derive(Serialize, serde::Deserialize)]
pub struct ScopeExport {
    pub scope: String,
    pub sources: Vec<provenance_core::Source>,
    pub requirements: Vec<provenance_core::Requirement>,
    pub resolutions: Vec<provenance_core::Resolution>,
    pub rules: Vec<provenance_core::Rule>,
    pub edges: Vec<provenance_core::Edge>,
    pub threads: Vec<provenance_core::Thread>,
    pub messages: Vec<provenance_core::Message>,
    #[serde(default)]
    pub contributions: Vec<provenance_core::Contribution>,
    #[serde(default)]
    pub synthesis_packets: Vec<provenance_core::SynthesisPacket>,
    #[serde(default)]
    pub proposal_cards: Vec<provenance_core::ProposalCard>,
    #[serde(default)]
    pub promotion_decisions: Vec<provenance_core::PromotionDecisionRecord>,
}

#[derive(Serialize)]
pub struct ImportReport {
    pub status: &'static str,
    pub dry_run: bool,
    pub records: usize,
}

pub fn export_scope(repo: Utf8PathBuf, scope: String) -> anyhow::Result<ScopeExport> {
    let scope_id = ScopeId::new(scope.clone())?;
    let store = StateStore::new(ProvenanceLayout::new(repo));
    Ok(ScopeExport {
        scope,
        sources: store.list_sources(&scope_id)?,
        requirements: store.list_requirements(&scope_id)?,
        resolutions: store.list_resolutions(&scope_id)?,
        rules: store.list_rules(&scope_id)?,
        edges: store
            .list_edges()?
            .into_iter()
            .filter(|edge| edge.scope_id == scope_id)
            .collect(),
        threads: store.list_threads(&scope_id)?,
        messages: store.list_messages(&scope_id)?,
        contributions: store.list_contributions(&scope_id)?,
        synthesis_packets: store.list_synthesis_packets(&scope_id)?,
        proposal_cards: store.list_proposal_cards(&scope_id)?,
        promotion_decisions: store.list_promotion_decisions(&scope_id)?,
    })
}

pub fn render_export(format: OutputFormat, exported: &ScopeExport) -> anyhow::Result<String> {
    match format {
        OutputFormat::Json => Ok(format!("{}\n", serde_json::to_string_pretty(exported)?)),
        OutputFormat::Jsonl => {
            let mut out = String::new();
            for value in serde_json::to_value(exported)?.as_object().unwrap().values() {
                if let Some(records) = value.as_array() {
                    for record in records {
                        out.push_str(&serde_json::to_string(record)?);
                        out.push('\n');
                    }
                }
            }
            Ok(out)
        }
        OutputFormat::Markdown => Ok(format!(
            "# Provenance Export\n\n- Scope: {}\n- Sources: {}\n- Requirements: {}\n- Resolutions: {}\n- Rules: {}\n- Edges: {}\n- Proposals: {}\n",
            exported.scope,
            exported.sources.len(),
            exported.requirements.len(),
            exported.resolutions.len(),
            exported.rules.len(),
            exported.edges.len(),
            exported.proposal_cards.len()
        )),
        OutputFormat::Toon => Ok(format!(
            "scope: {}\nsources: {}\nrequirements: {}\nresolutions: {}\nrules: {}\nedges: {}\nproposals: {}\n",
            exported.scope,
            exported.sources.len(),
            exported.requirements.len(),
            exported.resolutions.len(),
            exported.rules.len(),
            exported.edges.len(),
            exported.proposal_cards.len()
        )),
        OutputFormat::Table => Ok(format!(
            "scope\tsources\trequirements\tresolutions\trules\tedges\tproposals\n{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            exported.scope,
            exported.sources.len(),
            exported.requirements.len(),
            exported.resolutions.len(),
            exported.rules.len(),
            exported.edges.len(),
            exported.proposal_cards.len()
        )),
    }
}

pub fn import_scope(
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
    let records = exported.sources.len()
        + exported.requirements.len()
        + exported.resolutions.len()
        + exported.rules.len()
        + exported.edges.len()
        + exported.threads.len()
        + exported.messages.len()
        + exported.contributions.len()
        + exported.synthesis_packets.len()
        + exported.proposal_cards.len()
        + exported.promotion_decisions.len();
    if !dry_run {
        let layout = ProvenanceLayout::new(repo);
        let scope_id = ScopeId::new(scope)?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::sources_path(&layout, &scope_id),
            &exported.sources,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::requirements_path(&layout, &scope_id),
            &exported.requirements,
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
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::contributions_path(&layout, &scope_id),
            &exported.contributions,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::synthesis_packets_path(&layout, &scope_id),
            &exported.synthesis_packets,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::proposal_cards_path(&layout, &scope_id),
            &exported.proposal_cards,
        )?;
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::promotion_decisions_path(&layout, &scope_id),
            &exported.promotion_decisions,
        )?;
    }
    Ok(ImportReport {
        status: "ok",
        dry_run,
        records,
    })
}

pub fn coverage_scan(
    repo: Utf8PathBuf,
    path: &Utf8PathBuf,
    scope: String,
    validate_rules: bool,
) -> anyhow::Result<provenance_core::coverage::CoverageReport> {
    let scans = provenance_scanner::scan_path(path)?;
    let known_rules = if validate_rules {
        StateStore::new(ProvenanceLayout::new(repo))
            .list_rules(&ScopeId::new(scope)?)?
            .into_iter()
            .map(|rule| rule.rule_code)
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let scanner_warnings = if validate_rules {
        provenance_scanner::validate_annotations(&scans, known_rules.iter().cloned())
    } else {
        Vec::new()
    };
    let warnings = scanner_warnings
        .into_iter()
        .map(|warning| provenance_core::coverage::ValidationWarning {
            rule_code: warning.rule_code,
            file_path: warning.file_path,
            line: warning.line,
            message: warning.message,
        })
        .collect::<Vec<_>>();
    let annotations = scans
        .iter()
        .flat_map(|scan| &scan.annotations)
        .map(|location| provenance_core::coverage::AnnotationResult {
            rule_code: location.annotation.rule.clone(),
            file_path: location.file_path.clone(),
            line: location.line,
            function_name: location.function_name.clone(),
            coverage: location.annotation.coverage.to_string(),
            confidence: location.annotation.confidence,
        })
        .collect::<Vec<_>>();
    Ok(provenance_core::coverage::CoverageReport::new(
        current_git_commit().ok(),
        scans.len(),
        annotations,
        warnings,
    ))
}

pub fn current_git_commit() -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;
    anyhow::ensure!(output.status.success(), "git rev-parse failed");
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub fn render_coverage(
    format: OutputFormat,
    report: &provenance_core::coverage::CoverageReport,
) -> anyhow::Result<String> {
    if matches!(format, OutputFormat::Markdown) {
        let mut out = String::from("# Coverage Scan\n\n");
        writeln!(out, "- Files scanned: {}", report.files_scanned)?;
        writeln!(out, "- Total annotations: {}", report.total_annotations)?;
        writeln!(out, "- Warnings: {}\n", report.warnings.len())?;
        for annotation in &report.annotations {
            writeln!(
                out,
                "- `{}` in `{}`:{} ({})",
                annotation.rule_code, annotation.file_path, annotation.line, annotation.coverage
            )?;
        }
        for warning in &report.warnings {
            writeln!(
                out,
                "- Warning `{}` in `{}`:{}: {}",
                warning.rule_code, warning.file_path, warning.line, warning.message
            )?;
        }
        Ok(out)
    } else {
        Ok(serde_json::to_string_pretty(report)?)
    }
}

pub fn init(path: Utf8PathBuf, scope: String, path_prefix: Utf8PathBuf) -> anyhow::Result<()> {
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

pub fn check(repo: Utf8PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let manifest_path = ProvenanceLayout::new(repo).manifest_path();
    let manifest: Manifest = serde_json::from_str(&std::fs::read_to_string(manifest_path)?)?;
    anyhow::ensure!(
        !manifest.scopes.is_empty(),
        "manifest must contain at least one scope"
    );
    output::print(format, &Status { status: "ok" })
}
