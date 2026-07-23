//! Assembles the wiki page model from Provenance state.
//!
//! Pure joins over the scope export: edges are matched against record
//! vectors by stable id, in record order, so output is deterministic for a
//! given state. Every hole found on the way becomes a gap notice or an
//! orphan entry instead of being dropped.

mod context;
mod discovery;
mod evidence;
mod gaps;
mod page_links;
mod pages;
mod traversal;

use crate::handlers::ScopeExport;
use crate::wiki::links::{detect_remote_url, LinkResolver};
use crate::wiki::model::WikiCorpus;
use camino::Utf8PathBuf;
use context::Assembler;
use provenance_store::cache::{compute_gaps, GapGraph};

/// Loads the scope's state from disk and assembles the wiki corpus, using
/// the repo's `origin` remote (if any) to build evidence links.
pub fn load_corpus(repo: Utf8PathBuf, scope: String) -> anyhow::Result<WikiCorpus> {
    let remote_url = detect_remote_url(repo.as_std_path());
    let state = crate::handlers::export_scope(repo, scope)?;
    let resolver = LinkResolver::new(remote_url.as_deref());
    Ok(build_corpus(&state, &resolver))
}

/// Assembles the wiki corpus from already-loaded scope state.
pub fn build_corpus(state: &ScopeExport, resolver: &LinkResolver) -> WikiCorpus {
    let scope_id = provenance_core::ScopeId::new(&state.scope).expect("export scope is valid");
    let gaps = compute_gaps(&GapGraph {
        scope: &scope_id,
        sources: &state.sources,
        requirements: &state.requirements,
        resolutions: &state.resolutions,
        rules: &state.rules,
        topics: &state.topics,
        questions: &state.questions,
        edges: &state.edges,
        threads: &state.threads,
    });
    let assembler = Assembler {
        state,
        resolver,
        gaps: &gaps,
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
    let (domains, search) = discovery::build_discovery_pages(state, &requirements, &rules);
    WikiCorpus {
        scope: state.scope.clone(),
        index: assembler.index_page(),
        domains,
        search,
        requirements,
        resolutions,
        rules,
        sources: state
            .sources
            .iter()
            .map(|source| assembler.source_page(source))
            .collect(),
    }
}

#[cfg(test)]
mod tests;
