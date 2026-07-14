use crate::wiki::model::{PageId, RecordKind};
use camino::{Utf8Path, Utf8PathBuf};

pub(in crate::wiki) const WIKI_CSS_ROUTE: &str = "/assets/provenance-wiki.css";

pub(super) enum WikiRoute<'a> {
    Index,
    Topics,
    Search,
    Stylesheet,
    Record(&'a PageId),
}

impl WikiRoute<'_> {
    pub(super) fn path(self) -> String {
        match self {
            Self::Index => "/".to_string(),
            Self::Topics => "/topics/".to_string(),
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

pub(super) fn topic_anchor(domain_id: &str) -> String {
    format!("domain-{domain_id}")
}

pub(super) const UNASSIGNED_TOPIC_ANCHOR: &str = "topic-unassigned";

pub(super) fn topic_fragment(domain_id: &str) -> String {
    format!(
        "{}#{}",
        WikiRoute::Topics.path(),
        encode_fragment(&topic_anchor(domain_id))
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

pub(in crate::wiki) fn normalize_request_path(path: &str) -> String {
    let mut route = String::from("/");
    route.push_str(path.trim_matches('/'));
    if !route.ends_with('/') {
        route.push('/');
    }
    route
}

pub(in crate::wiki) fn static_page_path(out: &Utf8Path, route: &str) -> Utf8PathBuf {
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
    use crate::wiki::model::RecordKind;
    use camino::Utf8PathBuf;

    #[test]
    fn singleton_record_asset_and_topic_routes_have_one_authority() {
        assert_eq!(WikiRoute::Index.path(), "/");
        assert_eq!(WikiRoute::Topics.path(), "/topics/");
        assert_eq!(WikiRoute::Search.path(), "/search/");
        assert_eq!(WikiRoute::Stylesheet.path(), "/assets/provenance-wiki.css");
        for (kind, id, expected) in [
            (
                RecordKind::Requirement,
                "req_split",
                "/requirements/req_split/",
            ),
            (
                RecordKind::Resolution,
                "res_split",
                "/resolutions/res_split/",
            ),
            (RecordKind::Rule, "rule_split", "/rules/rule_split/"),
            (RecordKind::Source, "source_split", "/sources/source_split/"),
        ] {
            assert_eq!(WikiRoute::Record(&PageId::new(kind, id)).path(), expected);
        }
        assert_eq!(
            topic_fragment("domain/a b"),
            "/topics/#domain-domain%2Fa%20b"
        );
        assert_ne!(topic_anchor("unassigned"), UNASSIGNED_TOPIC_ANCHOR);
    }

    #[test]
    fn request_and_static_paths_share_canonical_route_rules() {
        assert_eq!(
            normalize_request_path("requirements/req_split"),
            "/requirements/req_split/"
        );
        assert_eq!(normalize_request_path("/"), "/");
        assert_eq!(
            static_page_path(&Utf8PathBuf::from("site"), "/requirements/req_split/"),
            Utf8PathBuf::from("site/requirements/req_split/index.html")
        );
    }
}
