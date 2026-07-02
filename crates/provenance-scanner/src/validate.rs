use std::collections::BTreeSet;

use camino::Utf8PathBuf;

use crate::walker::FileScan;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ValidationWarning {
    pub rule_code: String,
    pub file_path: Utf8PathBuf,
    pub line: usize,
    pub message: String,
}

pub fn validate_annotations(
    scans: &[FileScan],
    known_rule_codes: impl IntoIterator<Item = String>,
) -> Vec<ValidationWarning> {
    let known = known_rule_codes.into_iter().collect::<BTreeSet<_>>();
    let mut warnings = Vec::new();
    for scan in scans {
        for location in &scan.annotations {
            if !known.contains(&location.annotation.rule) {
                warnings.push(ValidationWarning {
                    rule_code: location.annotation.rule.clone(),
                    file_path: location.file_path.clone(),
                    line: location.line,
                    message: format!("unknown local rule code `{}`", location.annotation.rule),
                });
            }
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use camino::Utf8Path;

    use crate::walker::{scan_file, Language};

    use super::*;

    #[test]
    fn coverage_validation_warns_for_unknown_rule_code_with_location() {
        let scan = scan_file(
            Utf8Path::new("unknown_rule.rs"),
            Language::Rust,
            "// heading\n// @provenance rule: UNKNOWN-RULE\nfn test_rule() {}",
        );

        let warnings = validate_annotations(&[scan], ["SCHADS-PAY-001".to_string()]);

        assert_eq!(warnings[0].rule_code, "UNKNOWN-RULE");
        assert_eq!(warnings[0].file_path, Utf8Path::new("unknown_rule.rs"));
        assert_eq!(warnings[0].line, 2);
    }
}
