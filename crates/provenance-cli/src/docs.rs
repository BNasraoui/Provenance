use crate::output::{self, OutputFormat};
use anyhow::{bail, Context};
use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use camino::Utf8PathBuf;
use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use walkdir::WalkDir;

const DOCS_CSS: &str = r#"
:root {
  color-scheme: light dark;
  --pv-canvas-bg: #faf8f5;
  --pv-canvas-dots: #e0dbd4;
  --pv-card-bg: #fffdf9;
  --pv-card-border: #e4ddd3;
  --pv-card-shadow: rgba(120, 100, 70, 0.08);
  --pv-ink: #201b14;
  --pv-muted: #6f6659;
  --pv-source: #d4a574;
  --pv-requirement: #6b9e7a;
  --pv-resolution: #8b7bb5;
  --pv-rule: #5a8f9e;
  --pv-status-discovery: #b89e5a;
  --pv-font-display: "Source Serif 4", Georgia, "Times New Roman", serif;
  --pv-font-body: "DM Sans", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  --pv-font-mono: "DM Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

@media (prefers-color-scheme: dark) {
  :root {
    --pv-canvas-bg: #17140f;
    --pv-canvas-dots: #332b21;
    --pv-card-bg: #242119;
    --pv-card-border: #40382d;
    --pv-card-shadow: rgba(0, 0, 0, 0.3);
    --pv-ink: #f4ead9;
    --pv-muted: #b8a995;
    --pv-source: #d9ad78;
    --pv-requirement: #84bf91;
    --pv-resolution: #a898cf;
    --pv-rule: #77adba;
    --pv-status-discovery: #d1b16b;
  }
}

* { box-sizing: border-box; }

html { scroll-behavior: smooth; }

body {
  margin: 0;
  min-height: 100vh;
  color: var(--pv-ink);
  background:
    radial-gradient(circle at 12% 8%, color-mix(in srgb, var(--pv-resolution) 12%, transparent), transparent 26rem),
    radial-gradient(circle at 92% 18%, color-mix(in srgb, var(--pv-source) 14%, transparent), transparent 22rem),
    linear-gradient(180deg, var(--pv-canvas-bg), color-mix(in srgb, var(--pv-canvas-bg) 94%, var(--pv-card-border)));
  font-family: var(--pv-font-body);
  line-height: 1.65;
}

body::before {
  content: "";
  position: fixed;
  inset: 0;
  pointer-events: none;
  background-image: radial-gradient(var(--pv-canvas-dots) 0.75px, transparent 0.75px);
  background-size: 18px 18px;
  opacity: 0.35;
}

body > header {
  position: sticky;
  top: 0;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.8rem clamp(1rem, 3vw, 2rem);
  border-bottom: 1px solid var(--pv-card-border);
  background: color-mix(in srgb, var(--pv-canvas-bg) 88%, transparent);
  backdrop-filter: blur(16px) saturate(1.1);
}

body > header p { margin: 0; color: var(--pv-muted); font-size: 0.88rem; }
body > header strong { color: var(--pv-ink); font-family: var(--pv-font-display); font-size: 1.15rem; }

body > div {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: minmax(14rem, 18rem) minmax(0, 1fr);
  min-height: calc(100vh - 3.2rem);
}

aside {
  border-right: 1px solid var(--pv-card-border);
  padding: 1.25rem;
  background: color-mix(in srgb, var(--pv-card-bg) 76%, transparent);
}

aside nav { position: sticky; top: 5rem; }
nav h2 { margin: 0 0 0.75rem; color: var(--pv-muted); font-size: 0.72rem; letter-spacing: 0.12em; text-transform: uppercase; }
nav ol { list-style: none; margin: 0; padding: 0; }
nav li { margin: 0.12rem 0; }
nav li[data-depth="1"] { margin-left: 0.85rem; }
nav li[data-depth="2"] { margin-left: 1.7rem; }
nav li[data-depth="3"] { margin-left: 2.55rem; }
nav a {
  display: block;
  border-radius: 0.65rem;
  padding: 0.42rem 0.55rem;
  color: var(--pv-ink);
  text-decoration: none;
  font-size: 0.92rem;
}
nav a:hover { background: color-mix(in srgb, var(--pv-card-border) 50%, transparent); }
nav a[aria-current="page"] { color: var(--pv-resolution); background: color-mix(in srgb, var(--pv-resolution) 12%, transparent); font-weight: 700; }

main {
  width: min(100%, 78rem);
  padding: clamp(1.25rem, 4vw, 4rem) clamp(1rem, 5vw, 5rem) 5rem;
}

article {
  max-width: 52rem;
  border: 1px solid var(--pv-card-border);
  border-radius: 1.1rem;
  padding: clamp(1.2rem, 4vw, 3rem);
  background: color-mix(in srgb, var(--pv-card-bg) 93%, transparent);
  box-shadow: 0 1.4rem 4rem var(--pv-card-shadow);
}

article > p:first-child {
  display: inline-flex;
  width: auto;
  margin: 0 0 1rem;
  border: 1px solid color-mix(in srgb, var(--pv-status-discovery) 40%, transparent);
  border-radius: 999px;
  padding: 0.24rem 0.62rem;
  color: var(--pv-muted);
  background: color-mix(in srgb, var(--pv-status-discovery) 12%, transparent);
  font-size: 0.78rem;
  font-weight: 650;
}

h1, h2, h3 { font-family: var(--pv-font-display); line-height: 1.08; letter-spacing: -0.025em; }
h1 { margin: 0 0 1.15rem; max-width: 15ch; font-size: clamp(2.2rem, 7vw, 4.8rem); letter-spacing: -0.055em; }
h2 { margin-top: 2.1rem; font-size: 1.65rem; }
h3 { margin-top: 1.6rem; font-size: 1.25rem; }
p, li { color: color-mix(in srgb, var(--pv-ink) 86%, var(--pv-muted)); }
a { color: var(--pv-resolution); text-decoration-thickness: 0.08em; text-underline-offset: 0.18em; }
code, pre { font-family: var(--pv-font-mono); }
code { border-radius: 0.35rem; padding: 0.1rem 0.28rem; background: color-mix(in srgb, var(--pv-rule) 13%, transparent); }
pre { overflow: auto; border-radius: 0.85rem; padding: 1rem; background: color-mix(in srgb, var(--pv-card-border) 48%, transparent); }
blockquote { margin-left: 0; border-left: 0.28rem solid var(--pv-source); padding: 0.5rem 0 0.5rem 1rem; color: var(--pv-muted); }
table { width: 100%; border-collapse: collapse; }
th, td { border-bottom: 1px solid var(--pv-card-border); padding: 0.6rem 0.5rem; text-align: left; vertical-align: top; }
th { color: var(--pv-muted); font-size: 0.8rem; letter-spacing: 0.06em; text-transform: uppercase; }

@media (max-width: 820px) {
  body > header { align-items: flex-start; flex-direction: column; }
  body > div { grid-template-columns: 1fr; }
  aside { border-right: 0; border-bottom: 1px solid var(--pv-card-border); }
  aside nav { position: static; }
  main { padding-top: 1.25rem; }
}
"#;

#[derive(Debug, Clone)]
struct DocsSite {
    source_root: String,
    pages: Vec<DocPage>,
    route_by_path: BTreeMap<PathBuf, String>,
    page_by_route: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
struct DocPage {
    route: String,
    title: String,
    source_path: String,
    file_path: PathBuf,
    markdown: String,
    depth: usize,
}

#[derive(Serialize)]
struct DocsCheckReport {
    status: &'static str,
    source_root: String,
    homepage_route: &'static str,
    page_count: usize,
    pages: Vec<DocPageSummary>,
    broken_links: Vec<BrokenLink>,
}

#[derive(Serialize)]
struct DocPageSummary {
    route: String,
    title: String,
    source_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct BrokenLink {
    source_path: String,
    link: String,
    resolved_path: String,
}

pub fn check(repo: &Utf8PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let site = DocsSite::load(repo.as_std_path())?;
    site.ensure_links_valid()?;
    output::print(format, &site.check_report())?;
    Ok(())
}

pub async fn serve(repo: Utf8PathBuf, host: String, port: u16) -> anyhow::Result<()> {
    let site = DocsSite::load(repo.as_std_path())?;
    site.ensure_links_valid()?;
    let app = router(site);
    let listener = tokio::net::TcpListener::bind((host.as_str(), port))
        .await
        .with_context(|| format!("failed to bind docs server to {host}:{port}"))?;
    eprintln!(
        "serving Provenance docs at http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;
    Ok(())
}

impl DocsSite {
    fn load(repo: &Path) -> anyhow::Result<Self> {
        let repo = repo
            .canonicalize()
            .with_context(|| format!("repo path does not exist: {}", repo.display()))?;
        let docs_dir = repo.join("docs");
        let mut source_files = Vec::new();

        if docs_dir.is_dir() {
            for entry in WalkDir::new(&docs_dir).sort_by_file_name() {
                let entry = entry?;
                if entry.file_type().is_file() && is_markdown_file(entry.path()) {
                    source_files.push(entry.path().to_path_buf());
                }
            }
        }

        let has_docs_index = source_files
            .iter()
            .any(|path| path == &docs_dir.join("index.md"));
        let readme_path = repo.join("README.md");
        if !has_docs_index && readme_path.is_file() {
            source_files.push(readme_path);
        }

        if source_files.is_empty() {
            bail!("no docs found; create docs/index.md or README.md");
        }

        let source_root =
            if docs_dir.is_dir() && source_files.iter().any(|path| path.starts_with(&docs_dir)) {
                "docs"
            } else {
                "README.md"
            }
            .to_string();

        let mut pages = Vec::new();
        for file_path in source_files {
            let markdown = std::fs::read_to_string(&file_path)
                .with_context(|| format!("failed to read {}", file_path.display()))?;
            let source_path = slash_path(
                file_path
                    .strip_prefix(&repo)
                    .context("docs source was outside the repo")?,
            );
            let route = route_for_source(&repo, &docs_dir, &file_path)?;
            let title = infer_title(&markdown, &file_path);
            let depth = route_depth(&route);
            pages.push(DocPage {
                route,
                title,
                source_path,
                file_path: normalize_path(&file_path),
                markdown,
                depth,
            });
        }

        if pages.iter().all(|page| page.route != "/") {
            bail!("docs homepage not found; create docs/index.md or README.md");
        }

        pages.sort_by_key(page_sort_key);

        let mut route_to_source = BTreeMap::new();
        for page in &pages {
            if let Some(existing) =
                route_to_source.insert(page.route.clone(), page.source_path.clone())
            {
                bail!(
                    "docs route conflict for {} between {} and {}",
                    page.route,
                    existing,
                    page.source_path
                );
            }
        }

        let route_by_path = pages
            .iter()
            .map(|page| (page.file_path.clone(), page.route.clone()))
            .collect();
        let page_by_route = pages
            .iter()
            .enumerate()
            .map(|(index, page)| (page.route.clone(), index))
            .collect();

        Ok(Self {
            source_root,
            pages,
            route_by_path,
            page_by_route,
        })
    }

    fn check_report(&self) -> DocsCheckReport {
        DocsCheckReport {
            status: "ok",
            source_root: self.source_root.clone(),
            homepage_route: "/",
            page_count: self.pages.len(),
            pages: self
                .pages
                .iter()
                .map(|page| DocPageSummary {
                    route: page.route.clone(),
                    title: page.title.clone(),
                    source_path: page.source_path.clone(),
                })
                .collect(),
            broken_links: Vec::new(),
        }
    }

    fn ensure_links_valid(&self) -> anyhow::Result<()> {
        let broken_links = self.broken_links();
        if broken_links.is_empty() {
            return Ok(());
        }

        let mut message = format!("{} broken docs link(s)", broken_links.len());
        for link in broken_links.iter().take(10) {
            write!(
                message,
                "\n{} -> {} (resolved to {})",
                link.source_path, link.link, link.resolved_path
            )?;
        }
        bail!(message);
    }

    fn broken_links(&self) -> Vec<BrokenLink> {
        let mut broken = Vec::new();
        for page in &self.pages {
            for link in markdown_links(&page.markdown) {
                let Some(target_path) = resolve_markdown_link(page, &link) else {
                    continue;
                };
                if !self.route_by_path.contains_key(&target_path) {
                    broken.push(BrokenLink {
                        source_path: page.source_path.clone(),
                        link,
                        resolved_path: slash_path(&target_path),
                    });
                }
            }
        }
        broken
    }

    fn page_for_request_path(&self, path: &str) -> Option<&DocPage> {
        let route = normalize_request_route(path);
        self.page_by_route
            .get(&route)
            .and_then(|index| self.pages.get(*index))
    }

    fn render_page(&self, page: &DocPage) -> String {
        let nav = self.render_nav(&page.route);
        let body = self.render_markdown(page);
        format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{} - Provenance Docs</title>\n<link rel=\"stylesheet\" href=\"/assets/provenance-docs.css\">\n</head>\n<body>\n<header><p><strong>Provenance Docs</strong></p><p>plain Markdown, inferred routes, checked links</p></header>\n<div>\n<aside>{nav}</aside>\n<main><article><p><code>{}</code></p>{body}</article></main>\n</div>\n</body>\n</html>\n",
            escape_html(&page.title),
            escape_html(&page.source_path),
        )
    }

    fn render_not_found(&self, path: &str) -> String {
        let escaped_path = escape_html(path);
        let nav = self.render_nav("");
        format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Not Found - Provenance Docs</title>\n<link rel=\"stylesheet\" href=\"/assets/provenance-docs.css\">\n</head>\n<body>\n<header><p><strong>Provenance Docs</strong></p><p>plain Markdown, inferred routes, checked links</p></header>\n<div>\n<aside>{nav}</aside>\n<main><article><h1>Page not found</h1><p>No docs page is served at <code>{escaped_path}</code>.</p></article></main>\n</div>\n</body>\n</html>\n",
        )
    }

    fn render_nav(&self, current_route: &str) -> String {
        let mut nav = String::from("<nav aria-label=\"Docs navigation\"><h2>Docs</h2><ol>");
        for page in &self.pages {
            let current = if page.route == current_route {
                " aria-current=\"page\""
            } else {
                ""
            };
            let depth = page.depth.min(3);
            write!(
                nav,
                "<li data-depth=\"{depth}\"><a href=\"{}\"{current}>{}</a></li>",
                escape_attr(&page.route),
                escape_html(&page.title)
            )
            .expect("writing to a String should not fail");
        }
        nav.push_str("</ol></nav>");
        nav
    }

    fn render_markdown(&self, page: &DocPage) -> String {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_FOOTNOTES;
        let events = Parser::new_ext(&page.markdown, options).map(|event| match event {
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let rewritten = self
                    .rewrite_markdown_link(page, &dest_url)
                    .unwrap_or_else(|| dest_url.to_string());
                Event::Start(Tag::Link {
                    link_type,
                    dest_url: CowStr::Boxed(rewritten.into_boxed_str()),
                    title,
                    id,
                })
            }
            other => other,
        });
        let mut rendered = String::new();
        html::push_html(&mut rendered, events);
        rendered
    }

    fn rewrite_markdown_link(&self, page: &DocPage, destination: &str) -> Option<String> {
        let (_path_part, suffix) = local_markdown_path(destination)?;
        let target_path = resolve_markdown_link(page, destination)?;
        let route = self.route_by_path.get(&target_path)?;
        Some(format!("{route}{suffix}"))
    }
}

fn router(site: DocsSite) -> Router {
    Router::new()
        .route("/assets/provenance-docs.css", get(stylesheet))
        .fallback(get(render_docs_page))
        .with_state(Arc::new(site))
}

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        DOCS_CSS,
    )
}

async fn render_docs_page(State(site): State<Arc<DocsSite>>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().to_string();
    site.page_for_request_path(&path).map_or_else(
        || (StatusCode::NOT_FOUND, Html(site.render_not_found(&path))).into_response(),
        |page| Html(site.render_page(page)).into_response(),
    )
}

fn markdown_links(markdown: &str) -> Vec<String> {
    let mut links = Vec::new();
    for event in Parser::new(markdown) {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            if local_markdown_path(&dest_url).is_some() {
                links.push(dest_url.to_string());
            }
        }
    }
    links
}

fn resolve_markdown_link(page: &DocPage, destination: &str) -> Option<PathBuf> {
    let (path_part, _suffix) = local_markdown_path(destination)?;
    let target_path = page.file_path.parent()?.join(path_part);
    Some(normalize_path(&target_path))
}

fn local_markdown_path(destination: &str) -> Option<(&str, &str)> {
    if destination.starts_with('#') || destination.starts_with('/') || has_url_scheme(destination) {
        return None;
    }

    let (path_part, suffix) = destination.find('#').map_or((destination, ""), |index| {
        (&destination[..index], &destination[index..])
    });
    if path_part.is_empty() || !path_part.to_ascii_lowercase().ends_with(".md") {
        return None;
    }
    Some((path_part, suffix))
}

fn has_url_scheme(destination: &str) -> bool {
    let scheme_end = destination.find(':');
    let slash = destination.find('/');
    matches!((scheme_end, slash), (Some(colon), None) if colon > 0)
        || matches!((scheme_end, slash), (Some(colon), Some(slash)) if colon > 0 && colon < slash)
}

fn route_for_source(repo: &Path, docs_dir: &Path, file_path: &Path) -> anyhow::Result<String> {
    if file_path == repo.join("README.md") {
        return Ok("/".to_string());
    }

    let relative = file_path
        .strip_prefix(docs_dir)
        .with_context(|| format!("docs source is outside docs/: {}", file_path.display()))?;
    let mut parts = relative
        .with_extension("")
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    if parts.last().is_some_and(|part| part == "index") {
        parts.pop();
    }

    if parts.is_empty() {
        Ok("/".to_string())
    } else {
        Ok(format!("/{}/", parts.join("/")))
    }
}

fn normalize_request_route(path: &str) -> String {
    if path == "/" || path.is_empty() {
        return "/".to_string();
    }

    let mut route = path.to_string();
    if !route.starts_with('/') {
        route.insert(0, '/');
    }
    if !route.ends_with('/') {
        route.push('/');
    }
    route
}

fn page_sort_key(page: &DocPage) -> (u8, String) {
    if page.route == "/" {
        (0, String::new())
    } else {
        (1, page.source_path.clone())
    }
}

fn route_depth(route: &str) -> usize {
    route
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .count()
        .saturating_sub(1)
}

fn infer_title(markdown: &str, file_path: &Path) -> String {
    for line in markdown.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            let title = title.trim().trim_end_matches('#').trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }

    file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map_or_else(|| "Untitled".to_string(), titleize_slug)
}

fn titleize_slug(slug: &str) -> String {
    slug.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                format!("{}{}", first.to_uppercase(), chars.as_str())
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

fn slash_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            Component::RootDir => Some(String::new()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value).replace('"', "&quot;")
}
