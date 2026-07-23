//! Serves and builds the rendered wiki.
//!
//! Both entry points share one pipeline: [`crate::wiki::assemble::load_corpus`]
//! reads the scope's state, [`crate::wiki::render::render_corpus`] turns it
//! into route/HTML pairs, and this module either serves those pages over an
//! axum router (mirroring the docs server) or writes them out as a static
//! tree with the vendored stylesheet under `assets/`.

use crate::gitignore;
use crate::output::{self, OutputFormat};
use crate::wiki::assemble;
use crate::wiki::render::{self, RenderedPage};
use crate::wiki::routes::{normalize_request_path, static_page_path, WikiRoute, WIKI_CSS_ROUTE};
use crate::wiki::theme;
use anyhow::Context;
use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use camino::Utf8PathBuf;
use provenance_store::layout::ProvenanceLayout;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Pattern written to `.gitignore` for the default wiki output directory.
/// Trailing slash marks it as a directory, matching the existing
/// `.provenance/cache/` entry.
const WIKI_GITIGNORE_PATTERN: &str = ".provenance/wiki/";

struct WikiSite {
    scope: String,
    page_by_route: BTreeMap<String, RenderedPage>,
}

#[derive(Serialize)]
struct WikiBuildReport {
    status: &'static str,
    scope: String,
    out: Utf8PathBuf,
    page_count: usize,
    pages: Vec<WikiPageSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failures: Vec<WikiPageFailure>,
}

#[derive(Serialize)]
struct WikiPageSummary {
    route: String,
    title: String,
}

#[derive(Serialize)]
struct WikiPageFailure {
    route: String,
    error: String,
}

pub fn build(
    repo: Utf8PathBuf,
    scope: String,
    out: Option<Utf8PathBuf>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let out = if let Some(out) = out {
        out
    } else {
        gitignore::ensure_ignored(&repo, WIKI_GITIGNORE_PATTERN).with_context(|| {
            format!("failed to update .gitignore for the default wiki output at {repo}")
        })?;
        ProvenanceLayout::new(repo.clone()).wiki_dir()
    };
    let repo_hint = repo.clone();
    let corpus = assemble::load_corpus(repo, scope)?;
    let pages = render::render_corpus(&corpus);

    // Each page is written independently: a page-specific failure (a
    // colliding path, a permissions error) shouldn't hide every other page
    // that built fine. Failures are collected and reported instead.
    let mut failures = Vec::new();
    for page in &pages {
        if let Err(error) = write_page(&out, page) {
            failures.push(WikiPageFailure {
                route: page.route.clone(),
                error: format!("{error:#}"),
            });
        }
    }

    let stylesheet_path = out.join(WIKI_CSS_ROUTE.trim_start_matches('/'));
    let stylesheet_dir = stylesheet_path
        .parent()
        .with_context(|| format!("stylesheet path {stylesheet_path} has no parent directory"))?;
    std::fs::create_dir_all(stylesheet_dir)
        .with_context(|| format!("failed to create wiki output directory {stylesheet_dir}"))?;
    std::fs::write(&stylesheet_path, theme::WIKI_CSS)
        .with_context(|| format!("failed to write wiki stylesheet {stylesheet_path}"))?;

    let page_count = pages.len();
    let failure_count = failures.len();
    let report = WikiBuildReport {
        status: if failures.is_empty() { "ok" } else { "partial" },
        scope: corpus.scope,
        out,
        page_count,
        pages: pages
            .into_iter()
            .map(|page| WikiPageSummary {
                route: page.route,
                title: page.title,
            })
            .collect(),
        failures,
    };
    print_build_report(format, &report, &repo_hint)?;

    anyhow::ensure!(
        failure_count == 0,
        "{failure_count} of {page_count} wiki pages failed to write"
    );
    Ok(())
}

fn write_page(out: &Utf8PathBuf, page: &RenderedPage) -> anyhow::Result<()> {
    let path = static_page_path(out, &page.route);
    let parent = path
        .parent()
        .with_context(|| format!("wiki page path {path} has no parent directory"))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create wiki output directory {parent}"))?;
    std::fs::write(&path, &page.html)
        .with_context(|| format!("failed to write wiki page {path}"))?;
    Ok(())
}

/// JSON/JSONL get the full machine-readable report. The human-facing
/// formats (table, the CLI default, and friends) get a short summary
/// instead of a dump of every page -- with an explicit list of any pages
/// that failed to write.
fn print_build_report(
    format: OutputFormat,
    report: &WikiBuildReport,
    repo: &Utf8PathBuf,
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::Jsonl => output::print(format, report)?,
        OutputFormat::Table | OutputFormat::Markdown | OutputFormat::Toon => {
            let written = report.page_count - report.failures.len();
            let noun = if written == 1 { "page" } else { "pages" };
            println!(
                "Built {written} {noun} for scope \"{}\" into {}",
                report.scope, report.out
            );
            if report.failures.is_empty() {
                println!("Run `provenance wiki serve --repo {repo}` to view them.");
            } else {
                eprintln!("Failed to write {} page(s):", report.failures.len());
                for failure in &report.failures {
                    eprintln!("  - {}: {}", failure.route, failure.error);
                }
            }
        }
    }
    Ok(())
}

pub async fn serve(
    repo: Utf8PathBuf,
    scope: String,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let site = WikiSite::load(repo, scope)?;
    let app = router(site);
    let listener = tokio::net::TcpListener::bind((host.as_str(), port))
        .await
        .with_context(|| format!("failed to bind wiki server to {host}:{port}"))?;
    eprintln!(
        "serving Provenance wiki at http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;
    Ok(())
}

impl WikiSite {
    fn load(repo: Utf8PathBuf, scope: String) -> anyhow::Result<Self> {
        let corpus = assemble::load_corpus(repo, scope)?;
        let page_by_route = render::render_corpus(&corpus)
            .into_iter()
            .map(|page| (page.route.clone(), page))
            .collect();
        Ok(Self {
            scope: corpus.scope,
            page_by_route,
        })
    }

    fn page_for_request_path(&self, path: &str) -> Option<&RenderedPage> {
        self.page_by_route.get(&normalize_request_path(path))
    }
}

fn router(site: WikiSite) -> Router {
    Router::new()
        .route(&WikiRoute::Stylesheet.path(), get(stylesheet))
        .fallback(get(render_wiki_page))
        .with_state(Arc::new(site))
}

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        theme::WIKI_CSS,
    )
}

async fn render_wiki_page(State(site): State<Arc<WikiSite>>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().to_string();
    site.page_for_request_path(&path).map_or_else(
        || {
            (
                StatusCode::NOT_FOUND,
                Html(render::render_not_found(&site.scope, &path)),
            )
                .into_response()
        },
        |page| Html(page.html.clone()).into_response(),
    )
}
