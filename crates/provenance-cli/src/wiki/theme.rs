//! Vendored wiki stylesheet and theme switcher, ported from the design
//! mockup (`wiki-mockup-themes.html`). Everything is embedded in the
//! binary; generated pages make no external requests.
//!
//! The stylesheet is the mockup's `<style>` block verbatim, loaded as a
//! vendored asset so theme CSS stays out of Rust implementation files.

mod icons;
mod script;
mod search;

pub use icons::ICON_DEFS;
pub use script::THEME_SCRIPT;
pub use search::SEARCH_SCRIPT;

/// The full wiki stylesheet, served as one vendored asset.
pub const WIKI_CSS: &str = include_str!("theme/provenance-wiki.css");

#[cfg(test)]
mod tests;
