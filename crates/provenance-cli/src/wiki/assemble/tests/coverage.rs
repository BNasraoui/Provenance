use super::super::{build_corpus_with_coverage, repository_relative_path};
use super::fixtures::*;
use crate::wiki::links::LinkResolver;
use camino::Utf8PathBuf;
use provenance_core::coverage::{BindingResult, CoverageReport};

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
    }
}

#[test]
fn coverage_bindings_become_commit_pinned_rule_function_and_verification_sites() {
    let report = CoverageReport::new(
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
    );
    let resolver = LinkResolver::new(Some("git@github.com:exampleorg/ex-api.git"));

    let corpus = build_corpus_with_coverage(&fixture_state(), &resolver, Some(&report));
    let page = rule_page(&corpus, "rule_001");

    let function = page.rule_function.as_ref().unwrap();
    assert_eq!(function.symbol.as_deref(), Some("decide_rule"));
    assert_eq!(function.location.label, "src/rules.rs:7");
    assert_eq!(
        function.location.href.as_deref(),
        Some("https://github.com/exampleorg/ex-api/blob/abc1234/src/rules.rs#L7")
    );
    assert_eq!(page.verifications.len(), 2);
    assert!(!page.verifications[0].outside_defining_module);
    assert!(page.verifications[1].outside_defining_module);
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
