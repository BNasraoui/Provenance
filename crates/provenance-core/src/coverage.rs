use camino::Utf8PathBuf;

/// Something the scan wants to say about a rule.
///
/// `file_path` and `line` are `None` when the warning is about an absence.
/// An unverified rule has no site to point at, and naming one anyway sends a
/// reader to a file that says nothing about the problem.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ValidationWarning {
    pub rule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<Utf8PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AnnotationResult {
    pub rule_id: String,
    pub file_path: Utf8PathBuf,
    pub line: usize,
    pub function_name: Option<String>,
    pub coverage: String,
    pub confidence: f64,
}

/// A `#[rule]` or `#[verifies]` attribute site. `verification` is `None` for
/// a `#[rule]` site (the item is the rule) and the method word for a
/// `#[verifies]` site.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct BindingResult {
    pub rule_id: String,
    pub file_path: Utf8PathBuf,
    pub line: usize,
    pub item_name: Option<String>,
    pub verification: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CoverageReport {
    pub commit: Option<String>,
    pub files_scanned: usize,
    pub total_annotations: usize,
    pub warnings: Vec<ValidationWarning>,
    pub annotations: Vec<AnnotationResult>,
    pub bindings: Vec<BindingResult>,
}

impl CoverageReport {
    pub const fn new(
        commit: Option<String>,
        files_scanned: usize,
        annotations: Vec<AnnotationResult>,
        bindings: Vec<BindingResult>,
        warnings: Vec<ValidationWarning>,
    ) -> Self {
        Self {
            commit,
            files_scanned,
            total_annotations: annotations.len(),
            warnings,
            annotations,
            bindings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_report_counts_annotations() {
        let report = CoverageReport::new(
            Some("abc123".into()),
            2,
            vec![AnnotationResult {
                rule_id: "rule_overtime".into(),
                file_path: Utf8PathBuf::from("src/payroll.rs"),
                line: 4,
                function_name: Some("pays_overtime".into()),
                coverage: "full".into(),
                confidence: 1.0,
            }],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(report.total_annotations, 1);
    }
}
