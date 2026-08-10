use super::*;
use provenance_core::coverage::{CoverageReport, CoverageScan, ScannedFile};
use provenance_macros::verifies;
use std::fmt::Write as _;

fn scanned_resolver(path: &str) -> LinkResolver {
    scanned_resolver_at(path, "abcdef1")
}

fn scanned_resolver_at(path: &str, commit: &str) -> LinkResolver {
    let report = CoverageScan {
        report: CoverageReport::new(
            Some(commit.to_string()),
            1,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        scanned_files: vec![ScannedFile {
            file_path: path.into(),
            content: (1..=240).fold(String::new(), |mut content, line| {
                writeln!(content, "line {line}").unwrap();
                content
            }),
        }],
    };
    LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git")).with_coverage(&report)
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolve_document_suppresses_an_unpinned_lines_prefixed_section() {
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
    let evidence = resolver.resolve_document("src/UseCase.php", Some("lines 153-156"), None);
    assert_eq!(evidence.href.as_deref(), None);
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_passes_http_urls_through() {
    let evidence = LinkResolver::new(None).resolve("https://example.com/handbook");
    assert_eq!(evidence.label, "https://example.com/handbook");
    assert_eq!(
        evidence.href.as_deref(),
        Some("https://example.com/handbook")
    );
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_suppresses_unpinned_blob_urls_when_a_remote_is_known() {
    let evidence = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"))
        .resolve("src/UseCase.php:153-156");
    assert_eq!(evidence.label, "src/UseCase.php:153-156");
    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_pins_blob_urls_to_a_commit_when_given() {
    let evidence = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"))
        .resolve_at("src/UseCase.php:153", Some("deadbee"));
    assert_eq!(
        evidence.href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/deadbee/src/UseCase.php#L153")
    );
}

#[test]
fn resolver_suppresses_the_mutable_head_revision() {
    let evidence = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"))
        .resolve_at("src/UseCase.php:153", Some("HEAD"));

    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_leaves_code_refs_unlinked_without_a_remote() {
    let evidence = LinkResolver::new(None).resolve("src/UseCase.php:153-156");
    assert_eq!(evidence.label, "src/UseCase.php:153-156");
    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_leaves_prose_references_unlinked() {
    let evidence = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"))
        .resolve("Section 7.2 of the award");
    assert_eq!(evidence.label, "Section 7.2 of the award");
    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_leaves_bare_dotted_tokens_unlinked() {
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));
    for reference in ["e.g.", "Fig.", "etc.", "v1.2", "payroll.rs"] {
        let evidence = resolver.resolve(reference);
        assert_eq!(evidence.label, reference);
        assert!(
            evidence.href.is_none(),
            "`{reference}` linked to {:?}",
            evidence.href
        );
    }
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_suppresses_an_unpinned_bare_file_name_with_lines() {
    let evidence =
        LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git")).resolve("parser.rs:12");
    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_combines_documents_with_line_sections_without_linking() {
    let evidence = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"))
        .resolve_document("src/UseCase.php", Some("153-156"), None);
    assert_eq!(evidence.label, "src/UseCase.php:153-156");
    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_keeps_prose_sections_visible_without_an_unpinned_link() {
    let evidence = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"))
        .resolve_document("src/UseCase.php", Some("save flow"), None);
    assert_eq!(evidence.label, "src/UseCase.php (save flow)");
    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn resolver_leaves_prose_documents_unlinked() {
    let evidence = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"))
        .resolve_document("SCHADS Award", Some("clause 10.3"), None);
    assert_eq!(evidence.label, "SCHADS Award (clause 10.3)");
    assert!(evidence.href.is_none());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn annotate_links_file_references_in_free_text() {
    let resolver = scanned_resolver("src/UseCase.php");
    let text = "Pattern found in src/UseCase.php:153-156, per-portion guard.";
    let refs = resolver.annotate(text);
    assert_eq!(refs.len(), 1);
    assert_eq!(&text[refs[0].start..refs[0].end], "src/UseCase.php:153-156");
    assert_eq!(
        refs[0].href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/abcdef1/src/UseCase.php#L153-L156")
    );
    assert_eq!(
        refs[0].snippet.as_ref().unwrap().content,
        "line 153\nline 154\nline 155\nline 156"
    );
}

#[test]
fn snippets_separate_and_label_noncontiguous_ranges() {
    let evidence = scanned_resolver("src/UseCase.php").resolve("src/UseCase.php:10,100");
    let snippet = evidence.snippet.unwrap();

    assert_eq!(snippet.label, "src/UseCase.php:10, 100");
    assert_eq!(snippet.content, "line 10\n…\nline 100");
}

#[test]
fn annotate_keeps_spaced_noncontiguous_ranges_together() {
    let text = "See src/UseCase.php:10, 100 for both branches.";
    let refs = scanned_resolver("src/UseCase.php").annotate(text);

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].label, "src/UseCase.php:10, 100");
    assert_eq!(
        refs[0].snippet.as_ref().unwrap().content,
        "line 10\n…\nline 100"
    );
}

#[test]
fn long_snippets_label_the_lines_they_actually_show() {
    let evidence = scanned_resolver("src/UseCase.php").resolve("src/UseCase.php:10-30");
    let snippet = evidence.snippet.unwrap();

    assert_eq!(snippet.label, "src/UseCase.php:10-21 (requested 10-30)");
    assert!(snippet.content.ends_with("line 21\n…"));
}

#[test]
fn scanned_locations_beyond_the_file_are_not_linked() {
    let evidence = scanned_resolver("src/UseCase.php").resolve("src/UseCase.php:241");

    assert!(evidence.href.is_none());
    assert!(evidence.snippet.is_none());
}

#[test]
fn snippets_are_suppressed_when_the_link_targets_another_commit() {
    let evidence =
        scanned_resolver("src/UseCase.php").resolve_at("src/UseCase.php:241", Some("deadbee"));

    assert!(evidence.snippet.is_none());
    assert_eq!(
        evidence.href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/deadbee/src/UseCase.php#L241")
    );
}

#[test]
fn abbreviated_source_pins_match_the_full_scan_commit() {
    let resolver = scanned_resolver_at(
        "src/UseCase.php",
        "abcdef1234567890abcdef1234567890abcdef12",
    );
    let evidence = resolver.resolve_at("src/UseCase.php:10", Some("ABCDEF1"));

    assert!(evidence.snippet.is_some());
}

#[test]
fn snippets_need_an_explicit_line_location() {
    let evidence = scanned_resolver("src/UseCase.php").resolve("src/UseCase.php");

    assert!(evidence.snippet.is_none());
}

#[test]
fn annotate_links_test_case_names_to_the_nearby_file_reference() {
    let refs = scanned_resolver("src/UseCase.php")
        .annotate("src/UseCase.php:211-233 confirmed by testCreateGapInvoiceOnly.");
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[1].label, "testCreateGapInvoiceOnly");
    assert_eq!(
        refs[1].href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/abcdef1/src/UseCase.php")
    );
}

#[test]
fn annotate_links_a_leading_test_name_to_the_next_file_reference() {
    let refs = scanned_resolver("src/UseCase.php")
        .annotate("testCreateGapInvoiceOnly confirmed later at src/UseCase.php:211-233.");
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].label, "testCreateGapInvoiceOnly");
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn annotate_leaves_code_refs_unlinked_without_scan_data() {
    let resolver = LinkResolver::new(None);
    assert!(resolver
        .annotate("src/UseCase.php:211-233 confirmed by testCreateGapInvoiceOnly.")
        .is_empty());
}

#[test]
fn annotate_skips_test_names_without_a_file_reference() {
    assert!(scanned_resolver("src/UseCase.php")
        .annotate("Confirmed by testCreateGapInvoiceOnly.")
        .is_empty());
}

#[test]
#[verifies("rule_wiki_reference_links", examples)]
fn annotate_returns_nothing_for_plain_prose() {
    assert!(LinkResolver::new(None)
        .annotate("The award requires overtime pay.")
        .is_empty());
}

#[test]
fn annotate_leaves_unscanned_path_tokens_as_plain_text() {
    assert!(scanned_resolver("src/UseCase.php")
        .annotate("The partial/failure path remains under review.")
        .is_empty());
}
