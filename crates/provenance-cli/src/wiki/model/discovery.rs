use super::{PageId, PageKind, PageLink};
use serde::Serialize;

/// One requirement or rule in the static full-text index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchEntry {
    pub link: PageLink,
    pub kind: PageKind,
    pub statement: String,
}

/// Search data rendered into the offline page and emitted as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchIndexPage {
    pub id: PageId,
    pub scope: String,
    pub title: String,
    pub entries: Vec<SearchEntry>,
}

#[cfg(test)]
impl SearchIndexPage {
    /// Finds entries containing every query term, with title matches first.
    pub fn search(&self, query: &str) -> Vec<&SearchEntry> {
        let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        if terms.is_empty() {
            return Vec::new();
        }

        let mut matches: Vec<(usize, usize, &SearchEntry)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(position, entry)| {
                let title = entry.link.title.to_lowercase();
                let statement = entry.statement.to_lowercase();
                terms
                    .iter()
                    .all(|term| title.contains(term) || statement.contains(term))
                    .then(|| {
                        let score = terms
                            .iter()
                            .map(|term| if title.contains(term) { 2 } else { 1 })
                            .sum::<usize>()
                            + usize::from(title == query.trim().to_lowercase()) * 10;
                        (score, position, entry)
                    })
            })
            .collect();
        matches.sort_by_key(|(score, position, _)| (std::cmp::Reverse(*score), *position));
        matches.into_iter().map(|(_, _, entry)| entry).collect()
    }
}

/// A domain and all records whose requirement provenance places them there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopicGroup {
    pub domain_id: Option<String>,
    pub anchor: String,
    pub name: String,
    pub description: Option<String>,
    pub missing: bool,
    pub requirements: Vec<PageLink>,
    pub rules: Vec<PageLink>,
}

/// Domain/topic browsing index for a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopicIndexPage {
    pub id: PageId,
    pub scope: String,
    pub title: String,
    pub groups: Vec<TopicGroup>,
}
