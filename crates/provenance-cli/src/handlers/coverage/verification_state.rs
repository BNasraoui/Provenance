use std::collections::BTreeSet;

use provenance_core::ScopeId;
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};

pub(super) struct ValidationState {
    pub rules: Vec<provenance_core::Rule>,
    pub bindings: Vec<provenance_core::VerificationBinding>,
    pub warnings: Vec<provenance_core::coverage::ValidationWarning>,
}

pub(super) fn load_validation_state(
    repo: &camino::Utf8Path,
    scope: &str,
    scans: &[provenance_scanner::FileScan],
    enabled: bool,
) -> anyhow::Result<ValidationState> {
    if !enabled {
        return Ok(ValidationState {
            rules: Vec::new(),
            bindings: Vec::new(),
            warnings: Vec::new(),
        });
    }
    let store = StateStore::new(ProvenanceLayout::new(repo));
    let scope = ScopeId::new(scope)?;
    let rules = store.list_rules(&scope)?;
    let known = rules
        .iter()
        .map(|rule| rule.id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let mut warnings = provenance_scanner::validate_annotations(scans, known.iter().cloned());
    warnings.extend(provenance_scanner::validate_bindings(
        scans,
        known.iter().cloned(),
    ));
    Ok(ValidationState {
        rules,
        bindings: store.list_verification_bindings(&scope)?,
        warnings: warnings
            .into_iter()
            .map(|warning| provenance_core::coverage::ValidationWarning {
                rule_id: warning.rule_id,
                file_path: Some(warning.file_path),
                line: Some(warning.line),
                message: warning.message,
            })
            .collect(),
    })
}

/// Derives Unverified from both scanner sites and canonical typed bindings.
/// The finding carries no location because absence has no site to cite.
pub(super) fn unverified_rule_warnings(
    rules: &[provenance_core::Rule],
    scans: &[provenance_scanner::FileScan],
    typed_bindings: &[provenance_core::VerificationBinding],
) -> Vec<provenance_core::coverage::ValidationWarning> {
    let mut verified = scans
        .iter()
        .flat_map(|scan| {
            scan.bindings
                .iter()
                .filter(|binding| binding.verification.is_some())
                .map(|binding| binding.rule_id.clone())
                .chain(
                    scan.annotations
                        .iter()
                        .filter(|location| location.annotation.verification.is_some())
                        .map(|location| location.annotation.rule.clone()),
                )
        })
        .collect::<BTreeSet<_>>();
    verified.extend(
        typed_bindings
            .iter()
            .map(|binding| binding.rule_id.as_str().to_string()),
    );
    rules
        .iter()
        .filter(|rule| rule.status == provenance_core::RuleStatus::Active)
        .filter(|rule| !verified.contains(rule.id.as_str()))
        .map(|rule| provenance_core::coverage::ValidationWarning {
            rule_id: rule.id.as_str().to_string(),
            file_path: None,
            line: None,
            message: format!("active rule `{}` has no verification", rule.id.as_str()),
        })
        .collect()
}
