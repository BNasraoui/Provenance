//! Verification of `rule_page_route_safety`.
//!
//! Routes are unbounded strings, so the two properties below are checked
//! against generated routes rather than a fixed table. Both are stated
//! without reference to how `classify_route` takes a route apart: one joins
//! an accepted route onto a base directory and walks the result the way a
//! filesystem would, the other names the three refusals the decision itself
//! names and asserts every route carrying one is refused.

use provenance_macros::verifies;

use crate::wiki::model::{PageId, RecordKind};
use crate::wiki::publish::stage::{classify_route, RouteVerdict};
use crate::wiki::routes::WikiRoute;

/// `SplitMix64`. The seeds are fixed, so every run walks the same routes and
/// a failure is reproducible from the test name.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut word = self.0;
        word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        word ^ (word >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next_u64() % bound as u64).unwrap()
    }

    fn pick(&mut self, choices: &[&'static str]) -> &'static str {
        choices[self.below(choices.len())]
    }
}

/// Names a real page could plausibly carry, including ones that look like
/// traversal without being it.
const BENIGN_SEGMENTS: &[&str] = &[
    "requirements",
    "rules",
    "sources",
    "req_one",
    "rule_page_route_safety",
    "index.html",
    "a",
    "with-dash",
    "with.dot",
    "...",
    "....",
    "..a",
    "a..",
    "dot..dot",
    "unicode-é",
    "日本語",
    "%2e%2e",
    "..%2f..",
    "C:",
];

/// Names that either climb, hide a separator, or have nothing in them. No
/// NUL: it cannot survive most of the string plumbing a route travels
/// through, and it gets its own example below.
const HOSTILE_SEGMENTS: &[&str] = &[
    ".", "..", "", "/", "\\", "..\\..", "a\\b", "\\..", " ..", ".. ", "..;", "\u{1}", "\u{7f}",
    "\t", "\n", "\r",
];

/// Where the staged tree is rooted. An accepted route must land strictly
/// below these components, whatever it contains.
const BASE: [&str; 3] = ["tmp", "publish", "stage"];

fn gen_segment(rng: &mut Rng) -> &'static str {
    if rng.below(5) < 3 {
        rng.pick(BENIGN_SEGMENTS)
    } else {
        rng.pick(HOSTILE_SEGMENTS)
    }
}

/// Assembles a route from segments, then bounds it - properly, or with a
/// slash filed off either end, so the generator reaches routes that are not
/// routes at all.
fn gen_route(rng: &mut Rng) -> String {
    let count = rng.below(5);
    let segments: Vec<&str> = (0..count).map(|_| gen_segment(rng)).collect();
    let proper = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}/", segments.join("/"))
    };
    match rng.below(6) {
        3 => proper.strip_prefix('/').unwrap_or(&proper).to_string(),
        4 => proper.strip_suffix('/').unwrap_or(&proper).to_string(),
        5 => proper.trim_matches('/').to_string(),
        _ => proper,
    }
}

/// Joins the staged file path the publisher would write - one directory per
/// segment, then the page file - onto `BASE`, and walks it the way a
/// filesystem would, giving `.`, `..` and any separator hiding inside a
/// segment their real meaning back. `None` means the walk climbed out of
/// the base directory.
fn resolve<'a>(segments: &[&'a str]) -> Option<Vec<&'a str>> {
    let mut walked: Vec<&str> = BASE.to_vec();
    let components = segments
        .iter()
        .copied()
        .chain(std::iter::once("index.html"))
        .flat_map(|segment| segment.split(['/', '\\']));
    for component in components {
        match component {
            "" | "." => {}
            ".." => {
                if walked.len() <= BASE.len() {
                    return None;
                }
                walked.pop();
            }
            name => walked.push(name),
        }
    }
    Some(walked)
}

/// The three refusals the decision names, read straight off the route text.
fn stated_flaw(route: &str) -> Option<&'static str> {
    if !route.starts_with('/') {
        return Some("no leading slash");
    }
    if !route.ends_with('/') {
        return Some("no trailing slash");
    }
    // Between its bounding slashes a route is names separated by single
    // slashes, so two slashes in a row is a segment with no name in it.
    if route.len() > 1 && route.contains("//") {
        return Some("an empty segment");
    }
    if route.split('/').any(|segment| segment == "..") {
        return Some("a dot-dot segment");
    }
    None
}

#[test]
#[verifies("rule_page_route_safety", property)]
fn every_accepted_route_lands_inside_the_staging_directory() {
    let mut rng = Rng(0x5eed_0101);
    let mut accepted = 0;
    for _ in 0..4096 {
        let route = gen_route(&mut rng);
        let RouteVerdict::Safe(segments) = classify_route(&route) else {
            continue;
        };
        accepted += 1;

        // Accepting a route accepts it whole: the segments handed back
        // rebuild the route offered, so nothing was quietly dropped.
        let rebuilt = if segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}/", segments.join("/"))
        };
        assert_eq!(rebuilt, route, "accepted a reshaped route");

        let landed = resolve(&segments)
            .unwrap_or_else(|| panic!("accepted route climbed out of the stage: {route:?}"));
        assert!(
            landed.starts_with(&BASE),
            "accepted route left the stage: {route:?} landed at {landed:?}"
        );
        assert!(
            landed.len() > BASE.len(),
            "accepted route wrote the stage itself: {route:?}"
        );
        assert_eq!(landed.last(), Some(&"index.html"));
    }
    assert!(
        accepted >= 512,
        "only {accepted} generated routes were accepted; the property proves too little"
    );
}

#[test]
#[verifies("rule_page_route_safety", property)]
fn every_route_carrying_a_named_flaw_is_refused() {
    let mut rng = Rng(0x5eed_0202);
    let mut flawed = 0;
    for _ in 0..4096 {
        let route = gen_route(&mut rng);
        let Some(flaw) = stated_flaw(&route) else {
            continue;
        };
        flawed += 1;

        assert!(
            !matches!(classify_route(&route), RouteVerdict::Safe(_)),
            "accepted a route with {flaw}: {route:?}"
        );
    }
    assert!(
        flawed >= 512,
        "only {flawed} generated routes carried a named flaw; the property proves too little"
    );
}

#[test]
#[verifies("rule_page_route_safety", examples)]
fn refuses_the_traversal_shapes_the_staging_tests_drive() {
    for route in [
        "/../escaped/",
        "/requirements/../escape/",
        "/a/../../escaped/",
        "/./escaped/",
    ] {
        assert!(
            matches!(classify_route(route), RouteVerdict::UnsafeSegment(_)),
            "accepted a traversal route: {route:?}"
        );
    }

    // The escaping record id the rollback test plants mints exactly the
    // second route above.
    assert_eq!(
        WikiRoute::Record(&PageId::new(RecordKind::Requirement, "../escape")).path(),
        "/requirements/../escape/"
    );
}

#[test]
#[verifies("rule_page_route_safety", examples)]
fn refuses_routes_that_are_not_bounded_by_slashes() {
    assert_eq!(classify_route(""), RouteVerdict::Unbounded);
    assert_eq!(classify_route("requirements/"), RouteVerdict::Unbounded);
    assert_eq!(classify_route("/requirements"), RouteVerdict::Unbounded);
    assert_eq!(classify_route("requirements"), RouteVerdict::Unbounded);
}

#[test]
#[verifies("rule_page_route_safety", examples)]
fn refuses_segments_that_hide_further_path_structure() {
    assert_eq!(
        classify_route("/a\\..\\b/"),
        RouteVerdict::UnsafeSegment("a\\..\\b")
    );
    assert_eq!(
        classify_route("/a/\u{0}/b/"),
        RouteVerdict::UnsafeSegment("\u{0}")
    );
    assert_eq!(classify_route("/a//b/"), RouteVerdict::UnsafeSegment(""));
    assert_eq!(classify_route("//"), RouteVerdict::UnsafeSegment(""));
}

#[test]
#[verifies("rule_page_route_safety", examples)]
fn accepts_every_route_the_wiki_mints_for_a_page() {
    for route in [
        WikiRoute::Index.path(),
        WikiRoute::Domains.path(),
        WikiRoute::Search.path(),
        WikiRoute::Decisions.path(),
        WikiRoute::Unfinished.path(),
        WikiRoute::Record(&PageId::new(RecordKind::Rule, "rule_one")).path(),
        WikiRoute::Record(&PageId::new(RecordKind::Source, "src_one")).path(),
    ] {
        assert!(
            matches!(classify_route(&route), RouteVerdict::Safe(_)),
            "the wiki mints a page route the publisher refuses: {route:?}"
        );
    }
    assert_eq!(classify_route("/"), RouteVerdict::Safe(Vec::new()));
    assert_eq!(
        classify_route("/rules/rule_one/"),
        RouteVerdict::Safe(vec!["rules", "rule_one"])
    );

    // The stylesheet is an asset, not a page: it is written by name rather
    // than through a route, and this predicate would refuse it.
    assert_eq!(
        classify_route(&WikiRoute::Stylesheet.path()),
        RouteVerdict::Unbounded
    );
}
