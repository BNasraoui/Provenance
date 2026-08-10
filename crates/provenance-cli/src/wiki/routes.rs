//! Canonical route authority shared by rendering and serving.

use crate::wiki::model::{PageId, RecordKind};

pub const WIKI_CSS_ROUTE: &str = "/assets/provenance-wiki.css";
pub const UNASSIGNED_DOMAIN_ANCHOR: &str = "unassigned-domain-records";

pub enum WikiRoute<'a> {
    Index,
    Domains,
    Search,
    Stylesheet,
    Record(&'a PageId),
}

impl WikiRoute<'_> {
    /// Every variant but `Stylesheet` mints a page route, and a page route
    /// must satisfy `rule_page_route_safety`: bounded by slashes, with plain
    /// names between them. `Stylesheet` names an asset the publisher writes
    /// by name, not a page. Nothing here checks the record ids it is handed;
    /// the publisher refuses a route it cannot stage safely.
    pub fn path(self) -> String {
        match self {
            Self::Index => "/".to_string(),
            Self::Domains => "/domains/".to_string(),
            Self::Search => "/search/".to_string(),
            Self::Stylesheet => WIKI_CSS_ROUTE.to_string(),
            Self::Record(id) => {
                let collection = match id.kind {
                    RecordKind::Requirement => "requirements",
                    RecordKind::Resolution => "resolutions",
                    RecordKind::Rule => "rules",
                    RecordKind::Source => "sources",
                };
                format!("/{collection}/{}/", id.record_id)
            }
        }
    }
}

pub fn domain_anchor(domain_id: &str) -> String {
    format!("domain-{domain_id}")
}

pub fn domain_fragment(domain_id: &str) -> String {
    format!(
        "{}#{}",
        WikiRoute::Domains.path(),
        encode_fragment(&domain_anchor(domain_id))
    )
}

fn encode_fragment(fragment: &str) -> String {
    fragment.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(encoded, "%{byte:02X}").expect("writing to a String should not fail");
        }
        encoded
    })
}

/// Reshapes a request path into canonical route form so it can be looked up
/// among the pages already rendered. It vouches for nothing: `/../secrets/`
/// comes back unchanged and simply misses the page map, which is safe
/// because the lookup reads a map in memory and never the filesystem.
/// `rule_page_route_safety` governs the writing side alone.
pub fn normalize_request_path(path: &str) -> String {
    let mut route = String::from("/");
    route.push_str(path.trim_matches('/'));
    if !route.ends_with('/') {
        route.push('/');
    }
    route
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_routes_cover_singletons_records_and_requests() {
        assert_eq!(WikiRoute::Domains.path(), "/domains/");
        assert_eq!(WikiRoute::Search.path(), "/search/");
        assert_eq!(normalize_request_path("domains"), "/domains/");
        assert_eq!(normalize_request_path("/"), "/");
        assert_eq!(
            WikiRoute::Record(&PageId::new(RecordKind::Rule, "rule_one")).path(),
            "/rules/rule_one/"
        );
    }
}
