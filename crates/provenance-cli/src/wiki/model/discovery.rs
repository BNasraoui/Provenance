use super::{PageKind, PageLink};
use serde::Serialize;

/// One requirement or rule in the static full-text index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchEntry {
    pub link: PageLink,
    pub kind: PageKind,
    pub statement: String,
}

/// Search data rendered into the offline page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchIndexPage {
    pub scope: String,
    pub title: String,
    pub entries: Vec<SearchEntry>,
}

/// The semantic topic represented by a group. Each variant carries exactly
/// the metadata valid for that state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Topic {
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

/// A topic and all records whose requirement provenance places them there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopicGroup {
    pub topic: Topic,
    pub requirements: Vec<PageLink>,
    pub rules: Vec<PageLink>,
}

/// Domain/topic browsing index for a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopicIndexPage {
    pub scope: String,
    pub title: String,
    pub groups: Vec<TopicGroup>,
}
