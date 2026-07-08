use super::annotate::{is_test_name, tokens, trim_token};
use super::code_ref::{parse_code_ref, CodeRef};
use super::remote::{blob_url, parse_git_remote, GitRemote};
use serde::Serialize;

/// An evidence reference resolved for display: the original text plus a
/// link target when one could be derived. `href` is `None` for prose
/// references so the renderer shows them as plain text, never a dead link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceRef {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// A linkable span inside free text (byte offsets into the original text).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InlineRef {
    pub start: usize,
    pub end: usize,
    pub label: String,
    pub href: String,
}

/// Resolves evidence references against an optional git remote.
#[derive(Debug, Clone, Default)]
pub struct LinkResolver {
    remote: Option<GitRemote>,
}

impl LinkResolver {
    pub fn new(remote_url: Option<&str>) -> Self {
        Self {
            remote: remote_url.and_then(parse_git_remote),
        }
    }

    /// Resolves a reference at the default `HEAD` revision.
    pub fn resolve(&self, reference: &str) -> EvidenceRef {
        self.resolve_at(reference, None)
    }

    /// Resolves a reference pinned to a commit when one is known.
    pub fn resolve_at(&self, reference: &str, commit: Option<&str>) -> EvidenceRef {
        let label = reference.trim().to_string();
        if label.starts_with("http://") || label.starts_with("https://") {
            let href = Some(label.clone());
            return EvidenceRef { label, href };
        }
        let href = parse_code_ref(&label).and_then(|code_ref| self.href_for(&code_ref, commit));
        EvidenceRef { label, href }
    }

    /// Resolves a document/section pair, as stored on rules
    /// (`source_document` + `source_section`). A section that parses as line
    /// ranges is folded into the link anchor; a prose section stays visible
    /// in the label while the document alone is linked.
    pub fn resolve_document(
        &self,
        document: &str,
        section: Option<&str>,
        commit: Option<&str>,
    ) -> EvidenceRef {
        let Some(section) = section else {
            return self.resolve_at(document, commit);
        };
        let combined = format!("{document}:{section}");
        if let Some(code_ref) = parse_code_ref(&combined) {
            if !code_ref.lines.is_empty() {
                let href = self.href_for(&code_ref, commit);
                return EvidenceRef {
                    label: combined,
                    href,
                };
            }
        }
        let mut evidence = self.resolve_at(document, commit);
        evidence.label = format!("{document} ({section})");
        evidence
    }

    /// Finds linkable spans in free text: code references, and test-case
    /// names which link to the file of the nearest code reference in the
    /// same text. Bare dotted words without a separator or line group (for
    /// example `e.g.`) are left alone to avoid false positives.
    pub fn annotate(&self, text: &str) -> Vec<InlineRef> {
        let mut file_refs: Vec<(InlineRef, String)> = Vec::new();
        let mut test_names: Vec<InlineRef> = Vec::new();
        for (token_start, token) in tokens(text) {
            let (offset, trimmed) = trim_token(token);
            if trimmed.is_empty() {
                continue;
            }
            let start = token_start + offset;
            let end = start + trimmed.len();
            if let Some(code_ref) = parse_code_ref(trimmed) {
                if code_ref.path.contains('/') || !code_ref.lines.is_empty() {
                    // No remote means no link can be built at all; leave the
                    // token as plain text rather than anchor it to a path
                    // that would 404 inside the rendered site.
                    if let Some(href) = self.href_for(&code_ref, None) {
                        let file_href = self
                            .href_for(
                                &CodeRef {
                                    path: code_ref.path.clone(),
                                    lines: Vec::new(),
                                },
                                None,
                            )
                            .unwrap_or_default();
                        let inline = InlineRef {
                            start,
                            end,
                            label: trimmed.to_string(),
                            href,
                        };
                        file_refs.push((inline, file_href));
                    }
                    continue;
                }
            }
            if is_test_name(trimmed) {
                test_names.push(InlineRef {
                    start,
                    end,
                    label: trimmed.to_string(),
                    href: String::new(),
                });
            }
        }
        let mut refs: Vec<InlineRef> = Vec::new();
        for mut test_name in test_names {
            let nearest = file_refs
                .iter()
                .rev()
                .find(|(inline, _)| inline.start < test_name.start)
                .or_else(|| file_refs.first());
            if let Some((_, file_href)) = nearest {
                test_name.href.clone_from(file_href);
                refs.push(test_name);
            }
        }
        refs.extend(file_refs.into_iter().map(|(inline, _)| inline));
        refs.sort_by_key(|inline| inline.start);
        refs
    }

    /// Builds a blob URL for `code_ref` on the known remote. Returns `None`
    /// without a remote: a path like `src/UseCase.php` is relative to the
    /// repo root, not to the wiki page it would be rendered on, so it would
    /// always 404 as an `<a href>` inside the generated site.
    fn href_for(&self, code_ref: &CodeRef, commit: Option<&str>) -> Option<String> {
        self.remote
            .as_ref()
            .map(|remote| blob_url(remote, commit.unwrap_or("HEAD"), code_ref))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_document_anchors_a_lines_prefixed_section() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let evidence = resolver.resolve_document("src/UseCase.php", Some("lines 153-156"), None);
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L153-L156")
        );
    }

    #[test]
    fn resolver_passes_http_urls_through() {
        let resolver = LinkResolver::new(None);
        let evidence = resolver.resolve("https://example.com/handbook");
        assert_eq!(evidence.label, "https://example.com/handbook");
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://example.com/handbook")
        );
    }

    #[test]
    fn resolver_builds_blob_urls_when_a_remote_is_known() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let evidence = resolver.resolve("src/UseCase.php:153-156");
        assert_eq!(evidence.label, "src/UseCase.php:153-156");
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L153-L156")
        );
    }

    #[test]
    fn resolver_pins_blob_urls_to_a_commit_when_given() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let evidence = resolver.resolve_at("src/UseCase.php:153", Some("deadbee"));
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/deadbee/src/UseCase.php#L153")
        );
    }

    #[test]
    fn resolver_leaves_code_refs_unlinked_without_a_remote() {
        // A relative path like "src/UseCase.php" would be resolved against
        // the wiki page's own route, not the repo root, so it must never be
        // rendered as a real anchor when there is no remote to build a
        // proper blob URL from.
        let resolver = LinkResolver::new(None);
        let evidence = resolver.resolve("src/UseCase.php:153-156");
        assert_eq!(evidence.label, "src/UseCase.php:153-156");
        assert!(evidence.href.is_none());
    }

    #[test]
    fn resolver_leaves_prose_references_unlinked() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let evidence = resolver.resolve("Section 7.2 of the award");
        assert_eq!(evidence.label, "Section 7.2 of the award");
        assert!(evidence.href.is_none());
    }

    #[test]
    fn resolver_combines_documents_with_line_sections() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let evidence = resolver.resolve_document("src/UseCase.php", Some("153-156"), None);
        assert_eq!(evidence.label, "src/UseCase.php:153-156");
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L153-L156")
        );
    }

    #[test]
    fn resolver_keeps_prose_sections_visible_but_links_the_document() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let evidence = resolver.resolve_document("src/UseCase.php", Some("save flow"), None);
        assert_eq!(evidence.label, "src/UseCase.php (save flow)");
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php")
        );
    }

    #[test]
    fn resolver_leaves_prose_documents_unlinked() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let evidence = resolver.resolve_document("SCHADS Award", Some("clause 10.3"), None);
        assert_eq!(evidence.label, "SCHADS Award (clause 10.3)");
        assert!(evidence.href.is_none());
    }

    #[test]
    fn annotate_links_file_references_in_free_text() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let text = "Pattern found in src/UseCase.php:153-156, per-portion guard.";
        let refs = resolver.annotate(text);
        assert_eq!(refs.len(), 1);
        assert_eq!(&text[refs[0].start..refs[0].end], "src/UseCase.php:153-156");
        assert_eq!(refs[0].label, "src/UseCase.php:153-156");
        assert_eq!(
            refs[0].href,
            "https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L153-L156"
        );
    }

    #[test]
    fn annotate_links_test_case_names_to_the_nearby_file_reference() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let text = "src/UseCase.php:211-233 confirmed by testCreateGapInvoiceOnly.";
        let refs = resolver.annotate(text);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[1].label, "testCreateGapInvoiceOnly");
        assert_eq!(
            &text[refs[1].start..refs[1].end],
            "testCreateGapInvoiceOnly"
        );
        assert_eq!(
            refs[1].href,
            "https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php"
        );
    }

    #[test]
    fn annotate_links_a_leading_test_name_to_the_next_file_reference() {
        // When a test name appears before any file reference, there is no
        // "nearby" ref behind it, so annotate() falls back to the first
        // file reference found anywhere in the text.
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        let text = "testCreateGapInvoiceOnly confirmed later at src/UseCase.php:211-233.";
        let refs = resolver.annotate(text);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].label, "testCreateGapInvoiceOnly");
        assert_eq!(
            refs[0].href,
            "https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php"
        );
    }

    #[test]
    fn annotate_leaves_code_refs_unlinked_without_a_remote() {
        let resolver = LinkResolver::new(None);
        let text = "src/UseCase.php:211-233 confirmed by testCreateGapInvoiceOnly.";
        assert!(
            resolver.annotate(text).is_empty(),
            "no remote means no dead-link anchors for the file ref or the test name"
        );
    }

    #[test]
    fn annotate_skips_test_names_without_a_file_reference() {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        assert!(resolver
            .annotate("Confirmed by testCreateGapInvoiceOnly.")
            .is_empty());
    }

    #[test]
    fn annotate_returns_nothing_for_plain_prose() {
        let resolver = LinkResolver::new(None);
        assert!(resolver
            .annotate("The award requires overtime pay.")
            .is_empty());
    }
}
