use crate::wiki::render::routes::static_page_path;
use crate::wiki::render::RenderedPage;
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

const LEGACY_CSS: &str = include_str!("../theme/provenance-wiki-v0.css");
const CSS_PATH: &str = "assets/provenance-wiki.css";

const GLOBAL_NAV: &str = "<nav class=\"global-nav\" aria-label=\"Wiki\"><a href=\"/\">Atlas</a><a href=\"/topics/\">Topics</a><a href=\"/search/\">Search</a></nav>\n";
const DOMAIN_PREFIX: &str = "<div class=\"row\"><svg class=\"icon\"><use href=\"#i-git-branch\"/></svg><span class=\"k\">Domain</span><a class=\"v mono\" href=\"/topics/#";

/// Claims only byte-exact v0 pages reconstructed from the current corpus.
/// Caller files and stale pages that cannot be proven are intentionally kept.
pub(super) fn owned_files(
    stage: &Utf8Path,
    pages: &[RenderedPage],
) -> anyhow::Result<Vec<Utf8PathBuf>> {
    let css = stage.join(CSS_PATH);
    match std::fs::read_to_string(&css) {
        Ok(source) if source == LEGACY_CSS => {}
        Ok(_) => return Ok(Vec::new()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).with_context(|| format!("failed to read {css}")),
    }

    let mut owned = vec![Utf8PathBuf::from(CSS_PATH)];
    let mut has_root = false;
    for page in pages {
        let Some(expected) = as_legacy_html(page) else {
            continue;
        };
        let path = static_page_path(stage, &page.route);
        let relative = path.strip_prefix(stage)?;
        match std::fs::read_to_string(&path) {
            Ok(source) if source == expected => {
                has_root |= relative == Utf8Path::new("index.html");
                owned.push(relative.to_path_buf());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("failed to inspect {path}"));
            }
        }
    }
    if !has_root {
        return Ok(Vec::new());
    }
    Ok(owned)
}

fn as_legacy_html(page: &RenderedPage) -> Option<String> {
    if matches!(page.route.as_str(), "/topics/" | "/search/") {
        return None;
    }
    let html = page.html.strip_suffix('\n')?;
    let mut legacy = html.replace(GLOBAL_NAV, "");
    while let Some(start) = legacy.find(DOMAIN_PREFIX) {
        let href_start = start + DOMAIN_PREFIX.len();
        let link_start = legacy[href_start..].find("\">")? + href_start + 2;
        let link_end = legacy[link_start..].find("</a></div>")? + link_start;
        let value = legacy[link_start..link_end].to_string();
        legacy.replace_range(
            start..link_end + "</a></div>".len(),
            &format!(
                "<div class=\"row\"><svg class=\"icon\"><use href=\"#i-git-branch\"/></svg><span class=\"k\">Domain</span><span class=\"v mono\">{value}</span></div>"
            ),
        );
    }
    legacy.push('\n');
    Some(legacy)
}
