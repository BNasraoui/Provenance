use super::PageLink;
use serde::Serialize;

/// One requirement or rule in the offline full-text index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchEntry {
    pub link: PageLink,
    pub statement: String,
}

/// Search data rendered directly into the static page DOM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchIndexPage {
    pub scope: String,
    pub title: String,
    pub entries: Vec<SearchEntry>,
}

/// The metadata available for one reader-facing Domain group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DomainState {
    Defined {
        id: String,
        name: String,
        description: Option<String>,
    },
    Missing {
        id: String,
    },
    Unassigned,
}

/// A Domain and records placed there through requirement provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DomainGroup {
    pub state: DomainState,
    pub requirements: Vec<PageLink>,
    pub rules: Vec<PageLink>,
}

/// Reader taxonomy for one scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DomainIndexPage {
    pub scope: String,
    pub title: String,
    pub groups: Vec<DomainGroup>,
}
