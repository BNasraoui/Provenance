use crate::cli::workspace::CoverageCommand;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};
use std::collections::BTreeSet;
use std::fmt::Write;

pub(super) fn coverage_scan(
    repo: Utf8PathBuf,
    path: &Utf8PathBuf,
    scope: String,
    validate_rules: bool,
) -> anyhow::Result<provenance_core::coverage::CoverageReport> {
    let scans = provenance_scanner::scan_path(path)?;
    let rules = if validate_rules {
        StateStore::new(ProvenanceLayout::new(repo)).list_rules(&ScopeId::new(scope)?)?
    } else {
        Vec::new()
    };
    // Comment annotations cite rule codes; attributes cite rule ids. Accept
    // either name for either channel.
    let known_rules = rules
        .iter()
        .flat_map(|rule| [rule.rule_code.clone(), rule.id.as_str().to_string()])
        .collect::<BTreeSet<_>>();
    let mut scanner_warnings = if validate_rules {
        provenance_scanner::validate_annotations(&scans, known_rules.iter().cloned())
    } else {
        Vec::new()
    };
    if validate_rules {
        scanner_warnings.extend(provenance_scanner::validate_bindings(
            &scans,
            known_rules.iter().cloned(),
        ));
        scanner_warnings.extend(unverified_rule_warnings(&rules, &scans));
    }
    let warnings = scanner_warnings
        .into_iter()
        .map(|warning| provenance_core::coverage::ValidationWarning {
            rule_code: warning.rule_code,
            file_path: warning.file_path,
            line: warning.line,
            message: warning.message,
        })
        .collect::<Vec<_>>();
    let annotations = scans
        .iter()
        .flat_map(|scan| &scan.annotations)
        .map(|location| provenance_core::coverage::AnnotationResult {
            rule_code: location.annotation.rule.clone(),
            file_path: location.file_path.clone(),
            line: location.line,
            function_name: location.function_name.clone(),
            coverage: location.annotation.coverage.to_string(),
            confidence: location.annotation.confidence,
        })
        .collect::<Vec<_>>();
    let bindings = scans
        .iter()
        .flat_map(|scan| &scan.bindings)
        .map(|binding| provenance_core::coverage::BindingResult {
            rule_id: binding.rule_id.clone(),
            file_path: binding.file_path.clone(),
            line: binding.line,
            item_name: binding.item_name.clone(),
            verification: binding.verification.map(|method| method.to_string()),
        })
        .collect::<Vec<_>>();
    Ok(provenance_core::coverage::CoverageReport::new(
        current_git_commit().ok(),
        scans.len(),
        annotations,
        bindings,
        warnings,
    ))
}

/// An active rule with no verification site anywhere — no `#[verifies]`
/// attribute and no comment annotation carrying a `verification:` key — is
/// unverified. That state is reported, never stored.
fn unverified_rule_warnings(
    rules: &[provenance_core::Rule],
    scans: &[provenance_scanner::FileScan],
) -> Vec<provenance_scanner::ValidationWarning> {
    let verified = scans
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
    rules
        .iter()
        .filter(|rule| rule.status == provenance_core::RuleStatus::Active)
        .filter(|rule| !verified.contains(rule.id.as_str()) && !verified.contains(&rule.rule_code))
        .map(|rule| provenance_scanner::ValidationWarning {
            rule_code: rule.id.as_str().to_string(),
            file_path: rule
                .source_document
                .clone()
                .map_or_else(|| Utf8PathBuf::from(""), Utf8PathBuf::from),
            line: 0,
            message: format!("active rule `{}` has no verification", rule.id.as_str()),
        })
        .collect()
}

pub(super) fn current_git_commit() -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;
    anyhow::ensure!(output.status.success(), "git rev-parse failed");
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub(super) fn render_coverage(
    format: OutputFormat,
    report: &provenance_core::coverage::CoverageReport,
) -> anyhow::Result<String> {
    if matches!(format, OutputFormat::Markdown) {
        let mut out = String::from("# Coverage Scan\n\n");
        writeln!(out, "- Files scanned: {}", report.files_scanned)?;
        writeln!(out, "- Total annotations: {}", report.total_annotations)?;
        writeln!(out, "- Warnings: {}\n", report.warnings.len())?;
        for annotation in &report.annotations {
            writeln!(
                out,
                "- `{}` in `{}`:{} ({})",
                annotation.rule_code, annotation.file_path, annotation.line, annotation.coverage
            )?;
        }
        for binding in &report.bindings {
            let relation = binding.verification.as_ref().map_or_else(
                || "is the rule".to_string(),
                |method| format!("verified by {method}"),
            );
            writeln!(
                out,
                "- `{}` {} at `{}`:{}{}",
                binding.rule_id,
                relation,
                binding.file_path,
                binding.line,
                binding
                    .item_name
                    .as_deref()
                    .map(|name| format!(" ({name})"))
                    .unwrap_or_default()
            )?;
        }
        for warning in &report.warnings {
            writeln!(
                out,
                "- Warning `{}` in `{}`:{}: {}",
                warning.rule_code, warning.file_path, warning.line, warning.message
            )?;
        }
        Ok(out)
    } else {
        Ok(serde_json::to_string_pretty(report)?)
    }
}

pub(super) fn handle(command: CoverageCommand) -> anyhow::Result<()> {
    match command {
        CoverageCommand::Scan {
            repo,
            path,
            scope,
            validate_rules,
            strict,
            format,
            output,
        } => {
            let report = coverage_scan(repo, &path, scope, validate_rules)?;
            if let Some(output_path) = output {
                let rendered = render_coverage(format, &report)?;
                std::fs::write(output_path, rendered)?;
            } else if matches!(format, OutputFormat::Markdown | OutputFormat::Toon) {
                print!("{}", render_coverage(format, &report)?);
            } else {
                output::print(format, &report)?;
            }
            if strict && !report.warnings.is_empty() {
                anyhow::bail!(
                    "coverage scan found {} warning(s); rerun without --strict to inspect",
                    report.warnings.len()
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod unverified_rule_warnings_tests {
    use super::unverified_rule_warnings;
    use camino::Utf8PathBuf;
    use provenance_core::{Rule, RuleSeverity, RuleStatus, SchemaVersion, ScopeId, StableId};
    use provenance_scanner::{
        Annotation, AnnotationLocation, AttributeBinding, CoverageLevel, FileScan, Language,
        Verification,
    };

    fn rule(id: &str, rule_code: &str, status: RuleStatus) -> Rule {
        Rule {
            schema_version: SchemaVersion(1),
            scope_id: ScopeId::new("default").unwrap(),
            id: StableId::new(id).unwrap(),
            rule_code: rule_code.to_string(),
            name: None,
            description: None,
            statement: "Claims must be grouped by participant".to_string(),
            status,
            severity: RuleSeverity::High,
            rule_type: None,
            modality: None,
            confidence: None,
            extraction_method: None,
            source_document: None,
            source_section: None,
            origin_thread: None,
            origin_message: None,
        }
    }

    fn empty_scan() -> FileScan {
        FileScan {
            file_path: Utf8PathBuf::from("src/lib.rs"),
            language: Language::Rust,
            annotations: Vec::new(),
            bindings: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn scan_with_binding(rule_id: &str, verification: Option<Verification>) -> FileScan {
        FileScan {
            bindings: vec![AttributeBinding {
                file_path: Utf8PathBuf::from("src/lib.rs"),
                line: 1,
                item_name: Some("checks_it".to_string()),
                rule_id: rule_id.to_string(),
                verification,
            }],
            ..empty_scan()
        }
    }

    fn scan_with_annotation(rule_code: &str, verification: Option<Verification>) -> FileScan {
        FileScan {
            annotations: vec![AnnotationLocation {
                file_path: Utf8PathBuf::from("src/lib.rs"),
                line: 1,
                function_name: Some("checks_it".to_string()),
                annotation: Annotation {
                    rule: rule_code.to_string(),
                    name: None,
                    description: None,
                    tags: Vec::new(),
                    coverage: CoverageLevel::Full,
                    confidence: 1.0,
                    intent: None,
                    verification,
                },
            }],
            ..empty_scan()
        }
    }

    #[test]
    fn active_rule_with_no_verification_warns() {
        let active = rule("rule_foo", "FOO-001", RuleStatus::Active);

        let warnings = unverified_rule_warnings(&[active], &[]);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_code, "rule_foo");
        assert!(warnings[0].message.contains("has no verification"));
    }

    #[test]
    fn active_rule_verified_via_attribute_binding_does_not_warn() {
        let active = rule("rule_foo", "FOO-001", RuleStatus::Active);
        let scan = scan_with_binding("rule_foo", Some(Verification::Examples));

        let warnings = unverified_rule_warnings(&[active], std::slice::from_ref(&scan));

        assert!(warnings.is_empty());
    }

    #[test]
    fn active_rule_verified_via_comment_annotation_does_not_warn() {
        let active = rule("rule_foo", "FOO-001", RuleStatus::Active);
        let scan = scan_with_annotation("FOO-001", Some(Verification::Conformance));

        let warnings = unverified_rule_warnings(&[active], std::slice::from_ref(&scan));

        assert!(warnings.is_empty());
    }

    #[test]
    fn comment_annotation_without_a_verification_key_does_not_count() {
        let active = rule("rule_foo", "FOO-001", RuleStatus::Active);
        let scan = scan_with_annotation("FOO-001", None);

        let warnings = unverified_rule_warnings(&[active], std::slice::from_ref(&scan));

        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn draft_and_deprecated_rules_never_warn() {
        let draft = rule("rule_draft", "DRAFT-001", RuleStatus::Draft);
        let deprecated = rule("rule_deprecated", "DEP-001", RuleStatus::Deprecated);

        let warnings = unverified_rule_warnings(&[draft, deprecated], &[]);

        assert!(warnings.is_empty());
    }

    #[test]
    fn verification_matches_by_rule_code_as_well_as_rule_id() {
        let verified_by_code = rule("rule_by_code", "BY-CODE-001", RuleStatus::Active);
        let verified_by_id = rule("rule_by_id", "BY-ID-001", RuleStatus::Active);
        // The attribute channel cites the rule id even though the rule also
        // has a distinct rule_code; the comment channel cites the rule_code.
        let binding_scan = scan_with_binding("rule_by_id", Some(Verification::Property));
        let annotation_scan = scan_with_annotation("BY-CODE-001", Some(Verification::Property));

        let warnings = unverified_rule_warnings(
            &[verified_by_code, verified_by_id],
            &[binding_scan, annotation_scan],
        );

        assert!(warnings.is_empty());
    }

    #[test]
    fn active_rule_unverified_alongside_verified_rules_only_warns_once() {
        let verified = rule("rule_verified", "VERIFIED-001", RuleStatus::Active);
        let unverified = rule("rule_unverified", "UNVERIFIED-001", RuleStatus::Active);
        let scan = scan_with_binding("rule_verified", Some(Verification::Exhaustion));

        let warnings =
            unverified_rule_warnings(&[verified, unverified], std::slice::from_ref(&scan));

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_code, "rule_unverified");
    }
}
