//! Property tests for `rule_wiki_reference_links`.
//!
//! The set of strings someone can type into a reference field is unbounded,
//! so the decision cannot be checked by trying every input. These tests
//! build reference shapes from parts — web addresses, repo-relative paths
//! with and without line numbers, prose, and prose that looks path-ish —
//! cross them with every remote state a resolver can be built in, and check
//! the properties below. Each property is stated here from the generator's
//! own record of what it built, never by asking the resolver a second
//! question.

use super::LinkResolver;
use provenance_macros::verifies;

/// What the generator built, and therefore what the reader should get.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    /// Already a web address; a browser can follow it as it stands.
    WebAddress,
    /// Reads as a repo-relative file path — it carries a directory
    /// separator or a line group — so it links whenever a known code host
    /// can turn it into a real address.
    CodePath,
    /// A bare dotted token with no directory and no line group. `e.g.`,
    /// `Fig.` and `UseCase.php` all land here: nothing in the text tells
    /// the abbreviation from the file name, so none of them link on their
    /// own. Attaching a line group turns one into a `CodePath`.
    BareName,
    /// Anything else. Text a reader can search for, never a link.
    Prose,
}

struct Reference {
    text: String,
    shape: Shape,
}

const DIRECTORIES: [&str; 4] = ["", "src/", "src/App/Domain/", "docs/"];
const FILE_NAMES: [&str; 4] = [
    "UseCase.php",
    "save-invoice.md",
    "mod.rs",
    "PayrollHandler.java",
];
/// Every line-group form `parse_code_ref` accepts, plus the bare path.
const LINE_SUFFIXES: [&str; 6] = [
    "",
    ":153",
    ":153-156",
    ":lines 153-156",
    ":59\u{2013}69",
    ":168, 193, 218",
];

const SCHEMES: [&str; 2] = ["http://", "https://"];
const WEB_HOSTS: [&str; 2] = ["example.com", "intranet.example:8080"];
const WEB_PATHS: [&str; 4] = [
    "",
    "/handbook",
    "/handbook#section-7",
    "/repo/blob/HEAD/src/UseCase.php#L153",
];

/// Prose, including traps that carry dots, colons or a test-case name and
/// still must not be linked on any surface. `README` sits here rather than
/// with the bare names: it has no dot, so no line group can make a path of
/// it either.
const PROSE: [&str; 10] = [
    "Section 7.2 of the award",
    "The award requires overtime pay.",
    "Clause 10.3 of the enterprise agreement",
    "README",
    "12:30pm",
    "Confirmed by testCreateGapInvoiceOnly.",
    "e.g. the payroll handbook",
    "Mr. Smith signed off on the 4:30 handover",
    "version 1.2.3 of the spec",
    "Discussed at the Feb. 3 review",
];

/// Single dotted tokens: the abbreviations a writer drops into a sentence,
/// and a file name they are indistinguishable from.
///
/// The generator makes no claim about which is which, because the resolver
/// cannot make one either. Every name in `FILE_NAMES` also reaches this
/// shape through the path builder, with no directory and no line group.
const BARE_NAMES: [&str; 6] = ["e.g.", "i.e.", "Fig.", "etc.", "v1.2", "payroll.rs"];

/// Sections as they are stored on rules next to a document.
const SECTIONS: [&str; 4] = ["153-156", "lines 12-14", "save flow", "clause 10.3"];

const TEXT_PREFIXES: [&str; 4] = [
    "",
    "Pattern found in ",
    "The guard lives at ",
    "Confirmed by testCreateGapInvoiceOnly, see ",
];
const TEXT_SUFFIXES: [&str; 4] = [
    "",
    ".",
    ", per the per-portion guard.",
    " and testSecondCase covers the rest.",
];

/// The remote states a resolver can be built in: none at all, each host on
/// the allowlist, and a self-hosted host that is off it. `host_domain` is
/// the address prefix a reader should land on, and `None` marks the states
/// in which no code path can be linked.
#[derive(Clone, Copy)]
struct RemoteState {
    url: Option<&'static str>,
    host_domain: Option<&'static str>,
}

const REMOTE_STATES: [RemoteState; 5] = [
    RemoteState {
        url: None,
        host_domain: None,
    },
    RemoteState {
        url: Some("git@github.com:exampleorg/ex-api.git"),
        host_domain: Some("https://github.com/"),
    },
    RemoteState {
        url: Some("https://gitlab.com/group/subgroup/repo.git"),
        host_domain: Some("https://gitlab.com/"),
    },
    RemoteState {
        url: Some("ssh://git@bitbucket.org/team/repo.git"),
        host_domain: Some("https://bitbucket.org/"),
    },
    RemoteState {
        url: Some("https://git.internal.example/exampleorg/ex-api.git"),
        host_domain: None,
    },
];

/// Deterministic generator: the same seed always assembles the same
/// sentence, so a failure names the case that produced it.
struct Rng(u64);

impl Rng {
    const fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    fn below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bound = u64::try_from(bound).expect("bound fits in u64");
        usize::try_from((self.0 >> 33) % bound).expect("a value below bound fits in usize")
    }
}

fn references() -> Vec<Reference> {
    let mut refs = Vec::new();
    for directory in DIRECTORIES {
        for name in FILE_NAMES {
            for lines in LINE_SUFFIXES {
                // A directory separator or a line group makes it a path;
                // the bare name alone reads as prose, however file-like it
                // looks to a human.
                let shape = if directory.is_empty() && lines.is_empty() {
                    Shape::BareName
                } else {
                    Shape::CodePath
                };
                refs.push(Reference {
                    text: format!("{directory}{name}{lines}"),
                    shape,
                });
            }
        }
    }
    for text in BARE_NAMES {
        refs.push(Reference {
            text: text.to_string(),
            shape: Shape::BareName,
        });
    }
    for scheme in SCHEMES {
        for host in WEB_HOSTS {
            for path in WEB_PATHS {
                refs.push(Reference {
                    text: format!("{scheme}{host}{path}"),
                    shape: Shape::WebAddress,
                });
            }
        }
    }
    for text in PROSE {
        refs.push(Reference {
            text: text.to_string(),
            shape: Shape::Prose,
        });
    }
    refs
}

/// Wraps a reference in a sentence so the free-text surface sees it the way
/// a rationale field would carry it.
fn free_text(reference: &str, seed: u64) -> String {
    let mut rng = Rng::new(seed);
    let prefix = TEXT_PREFIXES[rng.below(TEXT_PREFIXES.len())];
    let suffix = TEXT_SUFFIXES[rng.below(TEXT_SUFFIXES.len())];
    format!("{prefix}{reference}{suffix}")
}

/// Every href the wiki would render for `reference` when nothing is
/// attached to it: the reference on its own, as a document with no section,
/// and as a span inside free text.
fn whole_field_hrefs(resolver: &LinkResolver, reference: &str) -> Vec<String> {
    let mut hrefs: Vec<String> = Vec::new();
    hrefs.extend(resolver.resolve(reference).href);
    hrefs.extend(resolver.resolve_at(reference, Some("deadbee")).href);
    hrefs.extend(resolver.resolve_document(reference, None, None).href);
    for seed in 0..4_u64 {
        hrefs.extend(
            resolver
                .annotate(&free_text(reference, seed))
                .into_iter()
                .filter_map(|inline| inline.href),
        );
    }
    hrefs
}

/// The hrefs the document/section surface renders, where a stored section
/// can supply the line group the reference itself lacks.
fn sectioned_hrefs(resolver: &LinkResolver, reference: &str) -> Vec<String> {
    SECTIONS
        .into_iter()
        .filter_map(|section| {
            resolver
                .resolve_document(reference, Some(section), None)
                .href
        })
        .collect()
}

/// Every href the wiki would render for `reference`, across all three
/// display surfaces.
fn emitted_hrefs(resolver: &LinkResolver, reference: &str) -> Vec<String> {
    let mut hrefs = whole_field_hrefs(resolver, reference);
    hrefs.extend(sectioned_hrefs(resolver, reference));
    hrefs
}

/// An address a browser can follow from any page: a scheme, then a host.
fn is_absolute_web_address(href: &str) -> bool {
    href.strip_prefix("https://")
        .or_else(|| href.strip_prefix("http://"))
        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
}

fn path_part(reference: &str) -> &str {
    reference
        .split_once(':')
        .map_or(reference, |(path, _)| path)
}

#[test]
#[verifies("rule_wiki_reference_links", property)]
fn every_emitted_href_is_an_absolute_web_address() {
    for state in REMOTE_STATES {
        let resolver = LinkResolver::new(state.url);
        for reference in references() {
            for href in emitted_hrefs(&resolver, &reference.text) {
                assert!(
                    is_absolute_web_address(&href),
                    "remote {:?} linked `{}` to `{href}`, which is not an absolute http(s) address",
                    state.url,
                    reference.text
                );
            }
        }
    }
}

#[test]
#[verifies("rule_wiki_reference_links", property)]
fn an_absent_or_unknown_remote_links_nothing_but_web_addresses() {
    for state in REMOTE_STATES {
        if state.host_domain.is_some() {
            continue;
        }
        let resolver = LinkResolver::new(state.url);
        for reference in references() {
            let hrefs = emitted_hrefs(&resolver, &reference.text);
            if reference.shape == Shape::WebAddress {
                assert!(
                    hrefs.iter().all(|href| *href == reference.text),
                    "remote {:?} rewrote the web address `{}` into {hrefs:?}",
                    state.url,
                    reference.text
                );
            } else {
                assert!(
                    hrefs.is_empty(),
                    "remote {:?} is not a known code host, yet `{}` linked to {hrefs:?}",
                    state.url,
                    reference.text
                );
            }
        }
    }
}

#[test]
#[verifies("rule_wiki_reference_links", property)]
fn prose_never_links_on_any_remote() {
    for state in REMOTE_STATES {
        let resolver = LinkResolver::new(state.url);
        for reference in references() {
            if reference.shape != Shape::Prose {
                continue;
            }
            let hrefs = emitted_hrefs(&resolver, &reference.text);
            assert!(
                hrefs.is_empty(),
                "remote {:?} linked the prose reference `{}` to {hrefs:?}",
                state.url,
                reference.text
            );
        }
    }
}

/// A dotted token standing on its own is prose on every surface, however
/// much it looks like a file name — and the same token becomes a file
/// reference as soon as a line group is attached to it.
#[test]
#[verifies("rule_wiki_reference_links", property)]
fn a_bare_dotted_name_links_only_once_a_line_group_is_attached() {
    for state in REMOTE_STATES {
        let resolver = LinkResolver::new(state.url);
        for reference in references() {
            if reference.shape != Shape::BareName {
                continue;
            }
            let hrefs = whole_field_hrefs(&resolver, &reference.text);
            assert!(
                hrefs.is_empty(),
                "remote {:?} linked the bare token `{}` to {hrefs:?}",
                state.url,
                reference.text
            );
            let Some(domain) = state.host_domain else {
                continue;
            };
            let href = resolver
                .resolve_document(&reference.text, Some("153-156"), None)
                .href
                .unwrap_or_else(|| {
                    panic!(
                        "remote {:?} is a known code host but left `{}` with a line \
                         section unlinked",
                        state.url, reference.text
                    )
                });
            assert!(
                href.starts_with(domain),
                "`{}` with a line section linked to `{href}`, which is not on {domain}",
                reference.text
            );
        }
    }
}

/// The other half of the decision: a link that would work is not withheld.
#[test]
#[verifies("rule_wiki_reference_links", property)]
fn a_known_code_host_links_every_file_path_to_itself() {
    for state in REMOTE_STATES {
        let Some(domain) = state.host_domain else {
            continue;
        };
        let resolver = LinkResolver::new(state.url);
        for reference in references() {
            if reference.shape != Shape::CodePath {
                continue;
            }
            let href = resolver.resolve(&reference.text).href.unwrap_or_else(|| {
                panic!(
                    "remote {:?} is a known code host but left `{}` unlinked",
                    state.url, reference.text
                )
            });
            assert!(
                href.starts_with(domain),
                "`{}` linked to `{href}`, which is not on {domain}",
                reference.text
            );
            assert!(
                href.contains(path_part(&reference.text)),
                "`{}` linked to `{href}`, which drops the path",
                reference.text
            );
        }
    }
}
