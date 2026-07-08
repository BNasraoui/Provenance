//! Vendored wiki stylesheet and theme switcher, ported from the design
//! mockup (`wiki-mockup-themes.html`). Everything is embedded in the
//! binary; generated pages make no external requests.
//!
//! The stylesheet is the mockup's `<style>` block verbatim -- five theme
//! token blocks (statesman default, piano, latte, mocha, dracula) plus the
//! theme-agnostic grammar -- followed by a small extensions section for the
//! page kinds and statuses the single mockup page did not need. All
//! extensions use the same `--pv-*` tokens, so every theme covers them.

mod assets;
mod css_renderer_extensions;
mod css_resolution_page;
mod css_theme_contract;

#[cfg(test)]
mod tests;

use std::sync::LazyLock;

pub use assets::{ICON_DEFS, THEME_SCRIPT};
use css_renderer_extensions::CSS_RENDERER_EXTENSIONS;
use css_resolution_page::CSS_RESOLUTION_PAGE;
use css_theme_contract::CSS_THEME_CONTRACT;

/// The full wiki stylesheet, served as one vendored asset.
pub static WIKI_CSS: LazyLock<String> = LazyLock::new(|| {
    let mut css = String::with_capacity(
        CSS_THEME_CONTRACT.len() + CSS_RESOLUTION_PAGE.len() + CSS_RENDERER_EXTENSIONS.len(),
    );
    css.push_str(CSS_THEME_CONTRACT);
    css.push_str(CSS_RESOLUTION_PAGE);
    css.push_str(CSS_RENDERER_EXTENSIONS);
    css
});

/// Returns the full wiki stylesheet as a single static string slice.
pub fn wiki_css() -> &'static str {
    WIKI_CSS.as_str()
}
