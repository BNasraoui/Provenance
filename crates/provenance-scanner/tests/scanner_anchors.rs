use camino::Utf8Path;
use provenance_scanner::{scan_file, Language};

#[test]
fn annotations_and_bindings_record_symbol_and_trimmed_line_hash() {
    let scan = scan_file(
        Utf8Path::new("rules.rs"),
        Language::Rust,
        "  // @provenance rule: rule_comment  \nfn comment_rule() {}\n\n  \
         #[rule(\"rule_attribute\")]  \nfn attribute_rule() {}\n",
    );
    let unpadded = scan_file(
        Utf8Path::new("rules.rs"),
        Language::Rust,
        "// @provenance rule: rule_comment\nfn comment_rule() {}\n\n\
         #[rule(\"rule_attribute\")]\nfn attribute_rule() {}\n",
    );

    assert_eq!(
        scan.annotations[0].anchor.symbol.as_deref(),
        Some("comment_rule")
    );
    assert_eq!(
        scan.bindings[0].anchor.symbol.as_deref(),
        Some("attribute_rule")
    );
    assert_eq!(
        scan.annotations[0].anchor.content_hash,
        unpadded.annotations[0].anchor.content_hash
    );
    assert_eq!(
        scan.bindings[0].anchor.content_hash,
        unpadded.bindings[0].anchor.content_hash
    );
    assert!(scan.bindings[0].anchor.content_hash.starts_with("sha256:"));
}
