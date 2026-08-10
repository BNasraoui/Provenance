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

pub fn validate_bindings(
    scans: &[FileScan],
    known_rule_ids: impl IntoIterator<Item = String>,
) -> Vec<ValidationWarning> {
    let known = known_rule_ids.into_iter().collect::<BTreeSet<_>>();
    let bindings = scans
        .iter()
        .flat_map(|scan| &scan.bindings)
        .collect::<Vec<_>>();
    let rule_sites = bindings
        .iter()
        .filter(|binding| binding.verification.is_none())
        .map(|binding| binding.rule_id.clone())
        .collect::<BTreeSet<_>>();
    let mut warnings = Vec::new();
    let mut seen_rule_sites = BTreeSet::new();
    for binding in bindings {
        if !known.contains(&binding.rule_id) {
            warnings.push(ValidationWarning {
                rule_code: binding.rule_id.clone(),
                file_path: binding.file_path.clone(),
                line: binding.line,
                message: format!("unknown rule id `{}`", binding.rule_id),
            });
        }
        if binding.verification.is_none() && !seen_rule_sites.insert(binding.rule_id.clone()) {
            warnings.push(ValidationWarning {
                rule_code: binding.rule_id.clone(),
                file_path: binding.file_path.clone(),
                line: binding.line,
                message: format!(
                    "more than one function carries #[rule(\"{}\")]; a rule is one function",
                    binding.rule_id
                ),
            });
        }
        if binding.verification.is_some() && !rule_sites.contains(&binding.rule_id) {
            warnings.push(ValidationWarning {
                rule_code: binding.rule_id.clone(),
                file_path: binding.file_path.clone(),
                line: binding.line,
                message: format!(
                    "#[verifies(\"{}\")] has no #[rule] function to verify",
                    binding.rule_id
                ),
            });
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
    fn binding_validation_warns_for_dangling_verifies_and_duplicate_rule() {
        let scan = scan_file(
            Utf8Path::new("rules.rs"),
            Language::Rust,
            "#[verifies(\"rule_orphaned\", examples)]\nfn checks_nothing() {}\n\
             #[rule(\"rule_twice\")]\nfn first() {}\n#[rule(\"rule_twice\")]\nfn second() {}",
        );

        let warnings = validate_bindings(
            &[scan],
            ["rule_orphaned".to_string(), "rule_twice".to_string()],
        );

        let messages = warnings
            .iter()
            .map(|w| w.message.as_str())
            .collect::<Vec<_>>();
        assert!(messages
            .iter()
            .any(|m| m.contains("no #[rule] function to verify")));
        assert!(messages
            .iter()
            .any(|m| m.contains("a rule is one function")));
    }

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
