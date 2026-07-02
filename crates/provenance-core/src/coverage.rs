use camino::Utf8PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ValidationWarning {
    pub rule_code: String,
    pub file_path: Utf8PathBuf,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AnnotationResult {
    pub rule_code: String,
    pub file_path: Utf8PathBuf,
    pub line: usize,
    pub function_name: Option<String>,
    pub coverage: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CoverageReport {
    pub commit: Option<String>,
    pub files_scanned: usize,
    pub total_annotations: usize,
    pub warnings: Vec<ValidationWarning>,
    pub annotations: Vec<AnnotationResult>,
}

impl CoverageReport {
    pub const fn new(
        commit: Option<String>,
        files_scanned: usize,
        annotations: Vec<AnnotationResult>,
        warnings: Vec<ValidationWarning>,
    ) -> Self {
        Self {
            commit,
            files_scanned,
            total_annotations: annotations.len(),
            warnings,
            annotations,
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
                rule_code: "SCHADS-PAY-001".into(),
                file_path: Utf8PathBuf::from("src/payroll.rs"),
                line: 4,
                function_name: Some("pays_overtime".into()),
                coverage: "full".into(),
                confidence: 1.0,
            }],
            Vec::new(),
        );

        assert_eq!(report.total_annotations, 1);
    }
}
