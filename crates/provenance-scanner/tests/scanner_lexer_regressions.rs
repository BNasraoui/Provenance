use std::collections::BTreeSet;

use camino::{Utf8Path, Utf8PathBuf};
use provenance_scanner::{scan_file, scan_path, Language};

#[test]
fn rust_lifetime_before_block_comment_does_not_hide_following_binding() {
    let source = r#"
fn f<'a>(x: u8) { let message = "it's valid"; /*
 * An unfinished raw-string example: r#"
 */ }

#[rule("real_rule")]
fn real_rule() {}
"#;
    let scan = scan_file(Utf8Path::new("fixture.rs"), Language::Rust, source);

    assert_eq!(scan.bindings.len(), 1);
    assert_eq!(scan.bindings[0].rule_id, "real_rule");
}

#[test]
fn rust_doc_marker_after_unmatched_double_quote_still_scans() {
    let source = r#"/// The input may contain an unmatched " before @provenance rule: real_rule
fn real_rule() {}"#;
    let scan = scan_file(Utf8Path::new("fixture.rs"), Language::Rust, source);

    assert_eq!(scan.annotations.len(), 1);
    assert_eq!(scan.annotations[0].annotation.rule, "real_rule");
}

#[test]
fn rust_doc_marker_after_lone_backtick_still_scans() {
    let source = "/// A Markdown ` before @provenance rule: real_rule\nfn real_rule() {}";
    let scan = scan_file(Utf8Path::new("fixture.rs"), Language::Rust, source);

    assert_eq!(scan.annotations.len(), 1);
    assert_eq!(scan.annotations[0].annotation.rule, "real_rule");
}

#[test]
fn go_comment_marker_inside_backticks_stays_hidden() {
    let source = "// Example: `@provenance rule: quoted_only`\nfunc realRule() {}";
    let scan = scan_file(Utf8Path::new("fixture.go"), Language::Go, source);

    assert!(scan.annotations.is_empty());
    assert!(scan.warnings.is_empty());
}

#[test]
fn store_scan_matches_every_rust_rule_and_verification_attribute() {
    let store = Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("scanner crate has a workspace parent")
        .join("provenance-store");
    let expected = grep_binding_sites(&store);
    let found = scan_path(&store)
        .expect("scan provenance-store")
        .into_iter()
        .flat_map(|scan| {
            scan.bindings.into_iter().map(|binding| {
                (
                    binding.file_path,
                    binding.line,
                    binding.verification.is_some(),
                )
            })
        })
        .collect::<BTreeSet<_>>();

    assert!(!expected.is_empty(), "dogfood corpus must contain bindings");
    assert_eq!(found, expected);
}

fn grep_binding_sites(root: &Utf8Path) -> BTreeSet<(Utf8PathBuf, usize, bool)> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .map(|entry| entry.expect("walk provenance-store"))
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| Utf8PathBuf::from_path_buf(entry.into_path()).ok())
        .filter(|path| path.extension() == Some("rs"))
        .flat_map(|path| {
            std::fs::read_to_string(&path)
                .expect("read Rust source")
                .lines()
                .enumerate()
                .filter_map(move |(line, text)| {
                    let text = text.trim_start();
                    if text.starts_with("#[rule(") {
                        Some((path.clone(), line + 1, false))
                    } else if text.starts_with("#[verifies(") {
                        Some((path.clone(), line + 1, true))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
