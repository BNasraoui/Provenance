use super::super::{build_corpus_with_coverage, repository_relative_path};
use super::fixtures::*;
use crate::wiki::links::LinkResolver;
use camino::Utf8PathBuf;
use provenance_core::coverage::{
    AnchorState, AnnotationResult, BindingResult, CoverageReport, CoverageScan, ScannedFile,
};
use std::fmt::Write as _;

fn binding(
    file_path: &str,
    line: usize,
    item_name: &str,
    verification: Option<&str>,
) -> BindingResult {
    BindingResult {
        rule_id: "rule_001".to_string(),
        file_path: Utf8PathBuf::from(file_path),
        line,
        item_name: Some(item_name.to_string()),
        verification: verification.map(str::to_string),
        anchor: None,
        anchor_state: AnchorState::Unchanged,
        original_line: None,
        original_file_path: None,
    }
}

fn annotation(
    file_path: &str,
    line: usize,
    function_name: &str,
    verification: Option<&str>,
) -> AnnotationResult {
    AnnotationResult {
        rule_id: "rule_001".to_string(),
        file_path: Utf8PathBuf::from(file_path),
        line,
        function_name: Some(function_name.to_string()),
        coverage: "full".to_string(),
        confidence: 1.0,
        verification: verification.map(str::to_string),
        anchor: None,
        anchor_state: AnchorState::Unchanged,
        original_line: None,
        original_file_path: None,
    }
}

#[test]
fn comment_annotations_become_implementation_and_verification_sites() {
    let report = CoverageScan {
        report: CoverageReport::new(
            Some("abc1234".to_string()),
            2,
            vec![
                annotation("src/rules.py", 7, "implement_rule", None),
                annotation("tests/test_rules.py", 12, "verify_rule", Some("examples")),
            ],
            Vec::new(),
            Vec::new(),
        ),
        scanned_files: Vec::new(),
    };
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));

    let corpus = build_corpus_with_coverage(&fixture_state(), &resolver, Some(&report));
    let page = rule_page(&corpus, "rule_001");

    assert_eq!(
        page.implementation.as_ref().unwrap().symbol.as_deref(),
        Some("implement_rule")
    );
    assert_eq!(page.verifications.len(), 1);
    assert_eq!(page.verifications[0].symbol.as_deref(), Some("verify_rule"));
    assert!(page.verifications[0].outside_implementation_module);
}

#[test]
fn coverage_bindings_become_commit_pinned_implementation_and_verification_sites() {
    let report = CoverageScan {
        report: CoverageReport::new(
            Some("abc1234".to_string()),
            2,
            Vec::new(),
            vec![
                binding("src/rules.rs", 7, "decide_rule", None),
                binding(
                    "src/rules.rs",
                    21,
                    "rule_holds_by_exhaustion",
                    Some("exhaustion"),
                ),
                binding(
                    "tests/rules.rs",
                    12,
                    "rule_holds_for_examples",
                    Some("examples"),
                ),
            ],
            Vec::new(),
        ),
        scanned_files: vec![ScannedFile {
            file_path: "src/UseCase.php".into(),
            content: (1..=200).fold(String::new(), |mut content, line| {
                writeln!(content, "line {line}").unwrap();
                content
            }),
        }],
    };
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));

    let corpus = build_corpus_with_coverage(&fixture_state(), &resolver, Some(&report));
    let page = rule_page(&corpus, "rule_001");

    let implementation = page.implementation.as_ref().unwrap();
    assert_eq!(implementation.symbol.as_deref(), Some("decide_rule"));
    assert_eq!(implementation.location.label, "src/rules.rs:7");
    assert_eq!(
        implementation.location.href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/abc1234/src/rules.rs#L7")
    );
    assert_eq!(page.verifications.len(), 2);
    assert!(!page.verifications[0].outside_implementation_module);
    assert!(page.verifications[1].outside_implementation_module);
    assert_eq!(
        page.code_scan.as_ref().unwrap().commit.as_deref(),
        Some("abc1234")
    );
    let requirement = requirement_page(&corpus, "req_child");
    assert_eq!(
        requirement.produced_rules[0].evidence[0].href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/abc1234/src/UseCase.php#L59-L69")
    );
    let references = requirement
        .threads
        .iter()
        .flat_map(|thread| &thread.messages)
        .flat_map(|message| &message.refs)
        .collect::<Vec<_>>();
    assert!(!references.is_empty());
    assert!(references.iter().all(|reference| reference
        .href
        .as_deref()
        .is_some_and(|href| href.contains("/blob/abc1234/"))));
}

/// A build given no report must leave `code_scan` unset, so the page can say
/// nothing was scanned instead of reporting an absent binding.
#[test]
fn a_corpus_built_without_a_report_records_no_code_scan() {
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));

    let corpus = build_corpus_with_coverage(&fixture_state(), &resolver, None);
    let page = rule_page(&corpus, "rule_001");

    assert!(page.code_scan.is_none());
    assert!(page.implementation.is_none());
    assert!(page.verifications.is_empty());
}

#[test]
fn gone_bindings_are_not_presented_as_current_code_sites() {
    let mut gone_rule = binding("src/rules.rs", 7, "decide_rule", None);
    gone_rule.anchor_state = AnchorState::Gone;
    let mut gone_verification = binding(
        "tests/rules.rs",
        12,
        "rule_holds_for_examples",
        Some("examples"),
    );
    gone_verification.anchor_state = AnchorState::Gone;
    let report = CoverageScan {
        report: CoverageReport::new(
            Some("abc1234".to_string()),
            2,
            Vec::new(),
            vec![gone_rule, gone_verification],
            Vec::new(),
        ),
        scanned_files: Vec::new(),
    };
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));

    let corpus = build_corpus_with_coverage(&fixture_state(), &resolver, Some(&report));
    let page = rule_page(&corpus, "rule_001");

    assert!(page.implementation.is_none());
    assert!(page.verifications.is_empty());
}

#[test]
fn an_uncommitted_scan_is_recorded_without_a_commit() {
    let report = CoverageScan {
        report: CoverageReport::new(
            None,
            1,
            Vec::new(),
            vec![binding("src/rules.rs", 7, "decide_rule", None)],
            Vec::new(),
        ),
        scanned_files: vec![ScannedFile {
            file_path: "src/rules.rs".into(),
            content: "one\ntwo\nthree\nfour\nfive\nsix\nfn decide_rule() {}\n".to_string(),
        }],
    };
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));

    let corpus = build_corpus_with_coverage(&fixture_state(), &resolver, Some(&report));
    let page = rule_page(&corpus, "rule_001");

    let scan = page.code_scan.as_ref().unwrap();
    assert!(scan.commit.is_none());
    let implementation = page.implementation.as_ref().unwrap();
    assert!(implementation.location.href.is_none());
    assert_eq!(
        implementation.location.snippet.as_ref().unwrap().content,
        "fn decide_rule() {}"
    );
}

#[test]
fn absolute_scan_paths_are_made_relative_to_the_canonical_repository() {
    let relative = repository_relative_path(
        camino::Utf8Path::new("/work/repo/src/rules.rs"),
        camino::Utf8Path::new("."),
        Some(camino::Utf8Path::new("/work/repo")),
    );

    assert_eq!(relative, Utf8PathBuf::from("src/rules.rs"));
}
