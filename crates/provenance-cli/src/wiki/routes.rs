//! Canonical route authority shared by rendering, static output, and serving.

use crate::wiki::model::{PageId, RecordKind};
use camino::{Utf8Path, Utf8PathBuf};

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

pub fn normalize_request_path(path: &str) -> String {
    let mut route = String::from("/");
    route.push_str(path.trim_matches('/'));
    if !route.ends_with('/') {
        route.push('/');
    }
    route
}

pub fn static_page_path(out: &Utf8Path, route: &str) -> Utf8PathBuf {
    let mut path = out.to_path_buf();
    for segment in route.split('/').filter(|segment| !segment.is_empty()) {
        path.push(segment);
    }
    path.push("index.html");
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serving_and_static_paths_follow_the_same_routes() {
        assert_eq!(normalize_request_path("domains"), "/domains/");
        assert_eq!(normalize_request_path("/"), "/");
        assert_eq!(
            static_page_path(&Utf8PathBuf::from("site"), "/domains/"),
            Utf8PathBuf::from("site/domains/index.html")
        );
    }
}
