//! Assembles the wiki page model from Provenance state.
//!
//! Pure joins over the scope export: edges are matched against record
//! vectors by stable id, in record order, so output is deterministic for a
//! given state. Every hole found on the way becomes a gap notice or an
//! orphan entry instead of being dropped.

mod context;
mod coverage;
mod discovery;
mod evidence;
mod gaps;
mod page_links;
mod pages;
mod traversal;

use crate::handlers::ScopeExport;
use crate::wiki::links::{detect_remote_url, LinkResolver};
use crate::wiki::model::WikiCorpus;
use anyhow::Context as _;
use camino::{Utf8Path, Utf8PathBuf};
use context::Assembler;
use provenance_core::coverage::CoverageReport;
use provenance_store::cache::{compute_gaps, GapGraph, GraphQuery};

/// Loads the scope's state from disk and assembles the wiki corpus, using
/// the repo's `origin` remote (if any) to build evidence links.
pub fn load_corpus(
    repo: Utf8PathBuf,
    scope: String,
    coverage: Option<&Utf8Path>,
) -> anyhow::Result<WikiCorpus> {
    let remote_url = detect_remote_url(repo.as_std_path());
    let coverage = coverage
        .map(|path| load_coverage_report(path, &repo))
        .transpose()?;
    let state = crate::handlers::export_scope(repo, scope)?;
    let resolver = LinkResolver::new(remote_url.as_deref());
    Ok(build_corpus_with_coverage(
        &state,
        &resolver,
        coverage.as_ref(),
    ))
}

/// Assembles the wiki corpus from already-loaded scope state.
#[cfg(test)]
pub fn build_corpus(state: &ScopeExport, resolver: &LinkResolver) -> WikiCorpus {
    build_corpus_with_coverage(state, resolver, None)
}

fn build_corpus_with_coverage(
    state: &ScopeExport,
    resolver: &LinkResolver,
    coverage: Option<&CoverageReport>,
) -> WikiCorpus {
    let scope_id = provenance_core::ScopeId::new(&state.scope).expect("export scope is valid");
    let graph = GapGraph {
        scope: &scope_id,
        sources: &state.sources,
        requirements: &state.requirements,
        resolutions: &state.resolutions,
        rules: &state.rules,
        topics: &state.topics,
        questions: &state.questions,
        edges: &state.edges,
        threads: &state.threads,
    };
    let gaps = compute_gaps(&graph);
    let assembler = Assembler {
        state,
        resolver,
        coverage,
        gaps: &gaps,
        query: GraphQuery::new(&graph),
        rule_requirements: std::cell::OnceCell::new(),
    };
    let requirements = state
        .requirements
        .iter()
        .map(|requirement| assembler.requirement_page(requirement))
        .collect::<Vec<_>>();
    let resolutions = state
        .resolutions
        .iter()
        .map(|resolution| assembler.resolution_page(resolution))
        .collect::<Vec<_>>();
    let rules = state
        .rules
        .iter()
        .map(|rule| assembler.rule_page(rule))
        .collect::<Vec<_>>();
    let sources = state
        .sources
        .iter()
        .map(|source| assembler.source_page(source))
        .collect::<Vec<_>>();
    let (domains, search) = discovery::build_discovery_pages(state, &requirements, &rules);
    let findings = assembler.findings_page();
    let index = assembler.index_page(&domains, &search, findings.findings.len());
    WikiCorpus {
        scope: state.scope.clone(),
        index,
        domains,
        search,
        findings,
        requirements,
        resolutions,
        rules,
        sources,
    }
}

fn load_coverage_report(path: &Utf8Path, repo: &Utf8Path) -> anyhow::Result<CoverageReport> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read coverage report {path}"))?;
    let mut report: CoverageReport = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse coverage report {path}"))?;
    let canonical_repo = std::fs::canonicalize(repo)
        .ok()
        .and_then(|path| Utf8PathBuf::from_path_buf(path).ok());
    for binding in &mut report.bindings {
        binding.file_path =
            repository_relative_path(&binding.file_path, repo, canonical_repo.as_deref());
    }
    Ok(report)
}

fn repository_relative_path(
    file_path: &Utf8Path,
    repo: &Utf8Path,
    canonical_repo: Option<&Utf8Path>,
) -> Utf8PathBuf {
    file_path
        .strip_prefix(repo)
        .or_else(|_| file_path.strip_prefix("."))
        .or_else(|error| canonical_repo.map_or(Err(error), |root| file_path.strip_prefix(root)))
        .unwrap_or(file_path)
        .to_path_buf()
}

#[cfg(test)]
mod tests;
