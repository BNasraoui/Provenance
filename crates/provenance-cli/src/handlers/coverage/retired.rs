//! Markers that cite a Rule the graph has retired.
//!
//! Unlike an unknown id, the record exists and its status is the reason the
//! marker cannot establish current coverage.

use std::collections::BTreeMap;

/// Markers that cite retired rules cannot establish current coverage. Unlike
/// an unknown id, the graph record exists and its status explains why the
/// marker is stale.
pub(super) fn stale_rule_warnings(
    rules: &[provenance_core::Rule],
    scans: &[provenance_scanner::FileScan],
) -> Vec<provenance_core::coverage::ValidationWarning> {
    let stale = rules
        .iter()
        .filter_map(|rule| {
            rule.retired
                .then_some("retired")
                .or_else(|| stale_status(&rule.status))
                .map(|status| (rule.id.as_str().to_string(), status))
        })
        .collect::<BTreeMap<_, _>>();

    scans
        .iter()
        .flat_map(|scan| {
            scan.annotations
                .iter()
                .filter_map(|location| {
                    stale.get(&location.annotation.rule).map(|status| {
                        stale_marker_warning(
                            &location.annotation.rule,
                            status,
                            location.file_path.clone(),
                            location.line,
                        )
                    })
                })
                .chain(scan.bindings.iter().filter_map(|binding| {
                    stale.get(&binding.rule_id).map(|status| {
                        stale_marker_warning(
                            &binding.rule_id,
                            status,
                            binding.file_path.clone(),
                            binding.line,
                        )
                    })
                }))
        })
        .collect()
}

const fn stale_status(status: &provenance_core::RuleStatus) -> Option<&'static str> {
    match status {
        provenance_core::RuleStatus::Deprecated => Some("deprecated"),
        provenance_core::RuleStatus::Archived => Some("archived"),
        _ => None,
    }
}

fn stale_marker_warning(
    rule_id: &str,
    status: &str,
    file_path: camino::Utf8PathBuf,
    line: usize,
) -> provenance_core::coverage::ValidationWarning {
    provenance_core::coverage::ValidationWarning {
        rule_id: rule_id.to_string(),
        file_path: Some(file_path),
        line: Some(line),
        message: format!("marker cites rule `{rule_id}` with status `{status}`"),
    }
}
