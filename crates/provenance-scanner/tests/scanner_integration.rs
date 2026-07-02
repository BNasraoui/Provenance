use camino::Utf8Path;
use provenance_scanner::{scan_file, CoverageLevel, Language};

#[test]
fn scans_python_decorator_annotation_on_decorated_function() {
    let scan = scan_file(
        Utf8Path::new("payroll.py"),
        Language::Python,
        r"# @provenance coverage: partial
# @provenance rule: SCHADS-PAY-001
@cached
def pays_overtime():
    pass
",
    );

    assert_eq!(scan.annotations.len(), 1);
    assert_eq!(
        scan.annotations[0].annotation.coverage,
        CoverageLevel::Partial
    );
    assert_eq!(
        scan.annotations[0].function_name.as_deref(),
        Some("pays_overtime")
    );
}

#[test]
fn scans_go_receiver_method_after_block_comment() {
    let scan = scan_file(
        Utf8Path::new("payroll.go"),
        Language::Go,
        r"/*
 * @provenance rule: SCHADS-PAY-002
 */
func (s *Service) PaysOvertime() bool { return true }
",
    );

    assert_eq!(scan.annotations.len(), 1);
    assert_eq!(
        scan.annotations[0].function_name.as_deref(),
        Some("PaysOvertime")
    );
}

#[test]
fn reports_malformed_directive_location() {
    let scan = scan_file(
        Utf8Path::new("payroll.rs"),
        Language::Rust,
        "// @provenance rule SCHADS-PAY-001\nfn pays_overtime() {}\n",
    );

    assert_eq!(scan.annotations.len(), 0);
    assert_eq!(scan.warnings[0].line, 1);
    assert!(scan.warnings[0].message.contains("malformed directive"));
}
