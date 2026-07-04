//! Pure link resolution for code and evidence references.
//!
//! Turns references such as `UseCase.php:153-156` into git host blob URLs
//! with line anchors when a remote is resolvable, or relative file links
//! otherwise. No IO except [`detect_remote_url`].

use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GitHost {
    GitHub,
    GitLab,
    Bitbucket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRemote {
    pub host: GitHost,
    pub owner: String,
    pub repo: String,
}

/// Parses an `origin` remote URL into a known git host, owner, and repo.
///
/// Accepts `https://`, `http://`, `ssh://`, and scp-style `git@host:path`
/// forms. Unknown hosts and local paths return `None`.
pub fn parse_git_remote(url: &str) -> Option<GitRemote> {
    let url = url.trim();
    let (host, path) = if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("ssh://"))
    {
        let rest = rest.split_once('@').map_or(rest, |(_, tail)| tail);
        rest.split_once('/')?
    } else if let Some(rest) = url.strip_prefix("git@") {
        rest.split_once(':')?
    } else {
        return None;
    };
    let host = match host {
        "github.com" | "www.github.com" => GitHost::GitHub,
        "gitlab.com" | "www.gitlab.com" => GitHost::GitLab,
        "bitbucket.org" | "www.bitbucket.org" => GitHost::Bitbucket,
        _ => return None,
    };
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path.rsplit_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some(GitRemote {
        host,
        owner: owner.to_string(),
        repo: repo.to_string(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LineRange {
    pub start: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u32>,
}

impl LineRange {
    pub const fn new(start: u32, end: Option<u32>) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeRef {
    pub path: String,
    pub lines: Vec<LineRange>,
}

/// Parses a code reference such as `src/UseCase.php:153-156`.
///
/// The path part must look like a file path (no whitespace, and either a
/// directory separator or a dotted file name). Line groups accept single
/// lines, `-`/en-dash ranges, and comma-separated lists.
pub fn parse_code_ref(text: &str) -> Option<CodeRef> {
    let text = text.trim();
    let (path, lines_part) = text
        .split_once(':')
        .map_or((text, None), |(path, lines)| (path, Some(lines)));
    if !is_file_path(path) {
        return None;
    }
    let lines = match lines_part {
        Some(lines_part) => parse_line_ranges(lines_part)?,
        None => Vec::new(),
    };
    Some(CodeRef {
        path: path.to_string(),
        lines,
    })
}

fn is_file_path(path: &str) -> bool {
    if path.is_empty() || path.contains("://") || path.chars().any(char::is_whitespace) {
        return false;
    }
    let file_name = path.rsplit('/').next().unwrap_or(path);
    path.contains('/') || file_name.contains('.')
}

/// Strips a leading "line"/"lines" word (case-insensitive), as in the
/// common human-written form `UseCase.php:lines 153-156`, so the numeric
/// parser underneath never has to know about it.
fn strip_leading_lines_word(part: &str) -> &str {
    let trimmed = part.trim_start();
    for word in ["lines", "line"] {
        if trimmed.len() > word.len()
            && trimmed.as_bytes()[word.len()].is_ascii_whitespace()
            && trimmed[..word.len()].eq_ignore_ascii_case(word)
        {
            return trimmed[word.len()..].trim_start();
        }
    }
    trimmed
}

fn parse_line_ranges(part: &str) -> Option<Vec<LineRange>> {
    let part = strip_leading_lines_word(part);
    part.split(',')
        .map(|group| {
            let group = group.trim();
            let (start, end) = group
                .split_once(['-', '\u{2013}'])
                .map_or((group, None), |(start, end)| {
                    (start.trim(), Some(end.trim()))
                });
            Some(LineRange::new(
                start.parse().ok()?,
                end.map(str::parse).transpose().ok()?,
            ))
        })
        .collect()
}

/// Builds a blob URL on the remote's host, anchored to the first line range.
pub fn blob_url(remote: &GitRemote, reference: &str, code_ref: &CodeRef) -> String {
    let anchor = code_ref
        .lines
        .first()
        .map_or_else(String::new, |range| line_anchor(remote.host, *range));
    let GitRemote { owner, repo, .. } = remote;
    let path = &code_ref.path;
    match remote.host {
        GitHost::GitHub => {
            format!("https://github.com/{owner}/{repo}/blob/{reference}/{path}{anchor}")
        }
        GitHost::GitLab => {
            format!("https://gitlab.com/{owner}/{repo}/-/blob/{reference}/{path}{anchor}")
        }
        GitHost::Bitbucket => {
            format!("https://bitbucket.org/{owner}/{repo}/src/{reference}/{path}{anchor}")
        }
    }
}

fn line_anchor(host: GitHost, range: LineRange) -> String {
    let LineRange { start, end } = range;
    match (host, end) {
        (GitHost::GitHub, Some(end)) => format!("#L{start}-L{end}"),
        (GitHost::GitLab, Some(end)) => format!("#L{start}-{end}"),
        (GitHost::Bitbucket, Some(end)) => format!("#lines-{start}:{end}"),
        (GitHost::GitHub | GitHost::GitLab, None) => format!("#L{start}"),
        (GitHost::Bitbucket, None) => format!("#lines-{start}"),
    }
}

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

fn tokens(text: &str) -> Vec<(usize, &str)> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (index, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if let Some(begin) = start.take() {
                tokens.push((begin, &text[begin..index]));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(begin) = start {
        tokens.push((begin, &text[begin..]));
    }
    tokens
}

fn trim_token(token: &str) -> (usize, &str) {
    const LEADING: &[char] = &['(', '[', '{', '"', '\'', '<'];
    const TRAILING: &[char] = &['.', ',', ';', ':', '!', '?', ')', ']', '}', '"', '\'', '>'];
    let trimmed = token.trim_start_matches(LEADING);
    let offset = token.len() - trimmed.len();
    (offset, trimmed.trim_end_matches(TRAILING))
}

fn is_test_name(token: &str) -> bool {
    token.strip_prefix("test").is_some_and(|rest| {
        rest.starts_with(|ch: char| ch.is_ascii_uppercase())
            && rest
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    })
}

/// Reads the `origin` remote URL of the repo at `repo`, if any.
pub fn detect_remote_url(repo: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!url.is_empty()).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_git_remote_reads_https_github_urls() {
        let remote = parse_git_remote("https://github.com/exampleorg/ex-api.git").unwrap();
        assert_eq!(remote.host, GitHost::GitHub);
        assert_eq!(remote.owner, "exampleorg");
        assert_eq!(remote.repo, "ex-api");
    }

    #[test]
    fn parse_git_remote_accepts_missing_git_suffix_and_trailing_slash() {
        let remote = parse_git_remote("https://github.com/exampleorg/ex-api/").unwrap();
        assert_eq!(remote.owner, "exampleorg");
        assert_eq!(remote.repo, "ex-api");
    }

    #[test]
    fn parse_git_remote_reads_scp_style_ssh_urls() {
        let remote = parse_git_remote("git@github.com:exampleorg/ex-api.git").unwrap();
        assert_eq!(remote.host, GitHost::GitHub);
        assert_eq!(remote.owner, "exampleorg");
        assert_eq!(remote.repo, "ex-api");
    }

    #[test]
    fn parse_git_remote_keeps_gitlab_subgroups_in_the_owner() {
        let remote = parse_git_remote("ssh://git@gitlab.com/group/subgroup/repo.git").unwrap();
        assert_eq!(remote.host, GitHost::GitLab);
        assert_eq!(remote.owner, "group/subgroup");
        assert_eq!(remote.repo, "repo");
    }

    #[test]
    fn parse_git_remote_reads_bitbucket_urls() {
        let remote = parse_git_remote("https://bitbucket.org/team/repo.git").unwrap();
        assert_eq!(remote.host, GitHost::Bitbucket);
        assert_eq!(remote.owner, "team");
        assert_eq!(remote.repo, "repo");
    }

    #[test]
    fn parse_git_remote_rejects_unknown_hosts_and_local_paths() {
        assert!(parse_git_remote("https://git.internal.example/owner/repo.git").is_none());
        assert!(parse_git_remote("/srv/git/repo.git").is_none());
        assert!(parse_git_remote("").is_none());
    }

    #[test]
    fn parse_code_ref_reads_a_plain_path() {
        let code_ref = parse_code_ref("docs/save-invoice.md").unwrap();
        assert_eq!(code_ref.path, "docs/save-invoice.md");
        assert!(code_ref.lines.is_empty());
    }

    #[test]
    fn parse_code_ref_reads_a_single_line() {
        let code_ref = parse_code_ref("UseCase.php:153").unwrap();
        assert_eq!(code_ref.path, "UseCase.php");
        assert_eq!(code_ref.lines, vec![LineRange::new(153, None)]);
    }

    #[test]
    fn parse_code_ref_reads_a_line_range() {
        let code_ref = parse_code_ref("UseCase.php:153-156").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(153, Some(156))]);
    }

    #[test]
    fn parse_code_ref_accepts_a_leading_lines_word() {
        let code_ref = parse_code_ref("UseCase.php:lines 153-156").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(153, Some(156))]);
    }

    #[test]
    fn parse_code_ref_accepts_a_leading_line_word_singular() {
        let code_ref = parse_code_ref("UseCase.php:line 42").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(42, None)]);
    }

    #[test]
    fn parse_code_ref_accepts_lines_word_case_insensitively_with_extra_space() {
        let code_ref = parse_code_ref("UseCase.php: Lines  59-69").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(59, Some(69))]);
    }

    #[test]
    fn resolve_document_anchors_a_lines_prefixed_section() {
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let evidence = resolver.resolve_document("src/UseCase.php", Some("lines 153-156"), None);
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156")
        );
    }

    #[test]
    fn parse_code_ref_accepts_en_dash_ranges() {
        let code_ref = parse_code_ref("UseCase.php:59\u{2013}69").unwrap();
        assert_eq!(code_ref.lines, vec![LineRange::new(59, Some(69))]);
    }

    #[test]
    fn parse_code_ref_reads_comma_separated_line_groups() {
        let code_ref = parse_code_ref("UseCase.php:168, 193, 218").unwrap();
        assert_eq!(
            code_ref.lines,
            vec![
                LineRange::new(168, None),
                LineRange::new(193, None),
                LineRange::new(218, None),
            ]
        );
    }

    #[test]
    fn parse_code_ref_rejects_prose_urls_and_bare_words() {
        assert!(parse_code_ref("Section 7.2 of the award").is_none());
        assert!(parse_code_ref("https://example.com/handbook").is_none());
        assert!(parse_code_ref("README").is_none());
        assert!(parse_code_ref("12:30pm").is_none());
        assert!(parse_code_ref("").is_none());
    }

    fn github_remote() -> GitRemote {
        GitRemote {
            host: GitHost::GitHub,
            owner: "exampleorg".to_string(),
            repo: "ex-api".to_string(),
        }
    }

    #[test]
    fn blob_url_anchors_github_line_ranges() {
        let code_ref = parse_code_ref("src/UseCase.php:153-156").unwrap();
        assert_eq!(
            blob_url(&github_remote(), "HEAD", &code_ref),
            "https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156"
        );
    }

    #[test]
    fn blob_url_anchors_github_single_lines() {
        let code_ref = parse_code_ref("src/UseCase.php:153").unwrap();
        assert_eq!(
            blob_url(&github_remote(), "abc1234", &code_ref),
            "https://github.com/exampleorg/ex-api/blob/abc1234/src/UseCase.php#L153"
        );
    }

    #[test]
    fn blob_url_uses_gitlab_dash_blob_layout() {
        let remote = GitRemote {
            host: GitHost::GitLab,
            owner: "group/subgroup".to_string(),
            repo: "repo".to_string(),
        };
        let code_ref = parse_code_ref("src/UseCase.php:59-69").unwrap();
        assert_eq!(
            blob_url(&remote, "HEAD", &code_ref),
            "https://gitlab.com/group/subgroup/repo/-/blob/HEAD/src/UseCase.php#L59-69"
        );
    }

    #[test]
    fn blob_url_uses_bitbucket_src_layout() {
        let remote = GitRemote {
            host: GitHost::Bitbucket,
            owner: "team".to_string(),
            repo: "repo".to_string(),
        };
        let code_ref = parse_code_ref("src/UseCase.php:153-156").unwrap();
        assert_eq!(
            blob_url(&remote, "HEAD", &code_ref),
            "https://bitbucket.org/team/repo/src/HEAD/src/UseCase.php#lines-153:156"
        );
    }

    #[test]
    fn blob_url_omits_the_anchor_without_lines() {
        let code_ref = parse_code_ref("docs/save-invoice.md").unwrap();
        assert_eq!(
            blob_url(&github_remote(), "HEAD", &code_ref),
            "https://github.com/exampleorg/ex-api/blob/HEAD/docs/save-invoice.md"
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
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let evidence = resolver.resolve("src/UseCase.php:153-156");
        assert_eq!(evidence.label, "src/UseCase.php:153-156");
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156")
        );
    }

    #[test]
    fn resolver_pins_blob_urls_to_a_commit_when_given() {
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let evidence = resolver.resolve_at("src/UseCase.php:153", Some("deadbee"));
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/exampleorg/ex-api/blob/deadbee/src/UseCase.php#L153")
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
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let evidence = resolver.resolve("Section 7.2 of the award");
        assert_eq!(evidence.label, "Section 7.2 of the award");
        assert!(evidence.href.is_none());
    }

    #[test]
    fn resolver_combines_documents_with_line_sections() {
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let evidence = resolver.resolve_document("src/UseCase.php", Some("153-156"), None);
        assert_eq!(evidence.label, "src/UseCase.php:153-156");
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156")
        );
    }

    #[test]
    fn resolver_keeps_prose_sections_visible_but_links_the_document() {
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let evidence = resolver.resolve_document("src/UseCase.php", Some("save flow"), None);
        assert_eq!(evidence.label, "src/UseCase.php (save flow)");
        assert_eq!(
            evidence.href.as_deref(),
            Some("https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php")
        );
    }

    #[test]
    fn resolver_leaves_prose_documents_unlinked() {
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let evidence = resolver.resolve_document("SCHADS Award", Some("clause 10.3"), None);
        assert_eq!(evidence.label, "SCHADS Award (clause 10.3)");
        assert!(evidence.href.is_none());
    }

    #[test]
    fn annotate_links_file_references_in_free_text() {
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let text = "Pattern found in src/UseCase.php:153-156, per-portion guard.";
        let refs = resolver.annotate(text);
        assert_eq!(refs.len(), 1);
        assert_eq!(&text[refs[0].start..refs[0].end], "src/UseCase.php:153-156");
        assert_eq!(refs[0].label, "src/UseCase.php:153-156");
        assert_eq!(
            refs[0].href,
            "https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php#L153-L156"
        );
    }

    #[test]
    fn annotate_links_test_case_names_to_the_nearby_file_reference() {
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
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
            "https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php"
        );
    }

    #[test]
    fn annotate_links_a_leading_test_name_to_the_next_file_reference() {
        // When a test name appears before any file reference, there is no
        // "nearby" ref behind it, so annotate() falls back to the first
        // file reference found anywhere in the text.
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
        let text = "testCreateGapInvoiceOnly confirmed later at src/UseCase.php:211-233.";
        let refs = resolver.annotate(text);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].label, "testCreateGapInvoiceOnly");
        assert_eq!(
            refs[0].href,
            "https://github.com/exampleorg/ex-api/blob/HEAD/src/UseCase.php"
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
        let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
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

    #[test]
    fn detect_remote_url_reads_the_origin_remote() {
        let dir = tempfile::tempdir().unwrap();
        let run = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(args)
                .status()
                .unwrap();
            assert!(status.success());
        };
        run(&["init", "--quiet"]);
        run(&[
            "remote",
            "add",
            "origin",
            "git@github.com:exampleorg/ex-api.git",
        ]);
        assert_eq!(
            detect_remote_url(dir.path()).as_deref(),
            Some("git@github.com:exampleorg/ex-api.git")
        );
    }

    #[test]
    fn detect_remote_url_returns_none_outside_a_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(detect_remote_url(dir.path()).is_none());
    }
}
