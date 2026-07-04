//! Serves and builds the rendered wiki.
//!
//! Both entry points share one pipeline: [`crate::wiki::assemble::load_corpus`]
//! reads the scope's state, [`crate::wiki::render::render_corpus`] turns it
//! into route/HTML pairs, and this module either serves those pages over an
//! axum router (mirroring the docs server) or writes them out as a static
//! tree with the vendored stylesheet under `assets/`.

use crate::output::{self, OutputFormat};
use crate::wiki::assemble;
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
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;

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
}

#[derive(Serialize)]
struct WikiPageSummary {
    route: String,
    title: String,
}

pub fn build(
    repo: Utf8PathBuf,
    scope: String,
    out: &Utf8PathBuf,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let corpus = assemble::load_corpus(repo, scope)?;
    let pages = render::render_corpus(&corpus);
    for page in &pages {
        let path = static_page_path(out, &page.route);
        let parent = path
            .parent()
            .with_context(|| format!("wiki page path {path} has no parent directory"))?;
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create wiki output directory {parent}"))?;
        std::fs::write(&path, &page.html)
            .with_context(|| format!("failed to write wiki page {path}"))?;
    }
    let stylesheet_path = out.join(WIKI_CSS_ROUTE.trim_start_matches('/'));
    let stylesheet_dir = stylesheet_path
        .parent()
        .with_context(|| format!("stylesheet path {stylesheet_path} has no parent directory"))?;
    std::fs::create_dir_all(stylesheet_dir)
        .with_context(|| format!("failed to create wiki output directory {stylesheet_dir}"))?;
    std::fs::write(&stylesheet_path, theme::WIKI_CSS)
        .with_context(|| format!("failed to write wiki stylesheet {stylesheet_path}"))?;

    let report = WikiBuildReport {
        status: "ok",
        scope: corpus.scope,
        out: out.clone(),
        page_count: pages.len(),
        pages: pages
            .into_iter()
            .map(|page| WikiPageSummary {
                route: page.route,
                title: page.title,
            })
            .collect(),
    };
    output::print(format, &report)?;
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

/// Maps a wiki route to its file in the static output tree: `/` becomes
/// `<out>/index.html`, `/requirements/<id>/` becomes
/// `<out>/requirements/<id>/index.html`.
fn static_page_path(out: &Utf8PathBuf, route: &str) -> Utf8PathBuf {
    let mut path = out.clone();
    for segment in route.split('/').filter(|segment| !segment.is_empty()) {
        path.push(segment);
    }
    path.push("index.html");
    path
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
    use super::{normalize_request_route, static_page_path};
    use camino::Utf8PathBuf;

    #[test]
    fn static_page_path_maps_index_route_to_root_index_html() {
        let out = Utf8PathBuf::from("site");
        assert_eq!(
            static_page_path(&out, "/"),
            Utf8PathBuf::from("site/index.html")
        );
    }

    #[test]
    fn static_page_path_maps_nested_routes_to_directory_index_html() {
        let out = Utf8PathBuf::from("site");
        assert_eq!(
            static_page_path(&out, "/requirements/req_sah/"),
            Utf8PathBuf::from("site/requirements/req_sah/index.html")
        );
    }

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
