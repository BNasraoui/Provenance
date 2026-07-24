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
use crate::wiki::publish::{self, PublicationOutput, PublishReport};
use crate::wiki::render::{self, RenderedPage, WIKI_CSS_ROUTE};
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

pub fn build(
    repo: Utf8PathBuf,
    scope: String,
    out: Option<Utf8PathBuf>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let output = if let Some(out) = out {
        PublicationOutput::custom(out)
    } else {
        gitignore::ensure_ignored(&repo, WIKI_GITIGNORE_PATTERN).with_context(|| {
            format!("failed to update .gitignore for the default wiki output at {repo}")
        })?;
        PublicationOutput::generator_owned(ProvenanceLayout::new(repo.clone()).wiki_dir())
    };
    let repo_hint = repo.clone();
    let corpus = assemble::load_corpus(repo, scope)?;
    let report = publish::publish(&corpus, output)?;
    print_build_report(format, &report, &repo_hint)?;
    Ok(())
}

/// JSON/JSONL get the full machine-readable report. The human-facing
/// formats (table, the CLI default, and friends) get a short summary
/// instead of a dump of every page -- with an explicit list of any pages
/// that failed to write.
fn print_build_report(
    format: OutputFormat,
    report: &PublishReport,
    repo: &Utf8PathBuf,
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::Jsonl => output::print(format, report)?,
        OutputFormat::Table | OutputFormat::Markdown | OutputFormat::Toon => {
            let written = report.page_count;
            let noun = if written == 1 { "page" } else { "pages" };
            println!(
                "Built {written} {noun} for scope \"{}\" into {}",
                report.scope, report.out
            );
            println!("Run `provenance wiki serve --repo {repo}` to view them.");
            for warning in &report.cleanup_warnings {
                eprintln!(
                    "Wiki publication committed, but cleanup requires attention at {}: {} ({})",
                    warning.path, warning.action, warning.error
                );
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
        self.page_by_route.get(&normalize_request_route(path))
    }
}

fn router(site: WikiSite) -> Router {
    Router::new()
        .route(WIKI_CSS_ROUTE, get(stylesheet))
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

fn normalize_request_route(path: &str) -> String {
    let mut route = String::from("/");
    route.push_str(path.trim_matches('/'));
    if !route.ends_with('/') {
        route.push('/');
    }
    route
}

#[cfg(test)]
mod tests {
    use super::normalize_request_route;

    #[test]
    fn normalize_request_route_adds_missing_slashes() {
        assert_eq!(
            normalize_request_route("/requirements/req_sah"),
            "/requirements/req_sah/"
        );
        assert_eq!(
            normalize_request_route("requirements/req_sah/"),
            "/requirements/req_sah/"
        );
        assert_eq!(normalize_request_route("/"), "/");
        assert_eq!(normalize_request_route(""), "/");
    }
}
