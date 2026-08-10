use crate::cli::workspace::CoverageCommand;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

pub(super) fn coverage_scan(
    repo: Utf8PathBuf,
    path: &Utf8PathBuf,
    scope: String,
    validate_rules: bool,
) -> anyhow::Result<provenance_core::coverage::CoverageReport> {
    let commit = current_git_commit(&repo).ok();
    let scans = provenance_scanner::scan_path(path)?;
    let rules = if validate_rules {
        StateStore::new(ProvenanceLayout::new(repo)).list_rules(&ScopeId::new(scope)?)?
    } else {
        Vec::new()
    };
    // Both channels — comment annotations and attributes — cite rule ids.
    let known_rules = rules
        .iter()
        .map(|rule| rule.id.as_str().to_string())
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
    }
    // Scanner warnings all come from a line the scan read, so each keeps its
    // location. Unverified rules are joined on after, without one.
    let mut warnings = parse_warnings(&scans);
    warnings.extend(scanner_warnings.into_iter().map(|warning| {
        provenance_core::coverage::ValidationWarning {
            rule_id: warning.rule_id,
            file_path: Some(warning.file_path),
            line: Some(warning.line),
            message: warning.message,
        }
    }));
    if validate_rules {
        warnings.extend(unverified_rule_warnings(&rules, &scans));
    }
    let annotations = scans
        .iter()
        .flat_map(|scan| &scan.annotations)
        .map(|location| provenance_core::coverage::AnnotationResult {
            rule_id: location.annotation.rule.clone(),
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
        commit,
        scans.len(),
        annotations,
        bindings,
        warnings,
    ))
}

/// What the parser complained about while reading the files: the legacy
/// Statesman marker, a directive with no `key: value`, a confidence
/// outside the range, an unknown field.
///
/// These name a file and a line but no rule. A malformed directive may never
/// get as far as saying which rule it meant, and inventing one would send a
/// reader after a rule that does not exist. The empty `rule_id` is what an
/// absent subject looks like, and the render leaves it out.
///
/// They are reported whether or not `--validate-rules` was passed: nothing
/// here is checked against the graph, so the scan owes them either way. Before
/// this, every one of them was dropped on the floor.
fn parse_warnings(
    scans: &[provenance_scanner::FileScan],
) -> Vec<provenance_core::coverage::ValidationWarning> {
    scans
        .iter()
        .flat_map(|scan| {
            scan.warnings
                .iter()
                .map(|warning| provenance_core::coverage::ValidationWarning {
                    rule_id: String::new(),
                    file_path: Some(scan.file_path.clone()),
                    line: Some(warning.line),
                    message: warning.message.clone(),
                })
        })
        .collect()
}

/// An active rule with no verification site anywhere — no `#[verifies]`
/// attribute and no comment annotation carrying a `verification:` key — is
/// unverified. That state is reported, never stored.
///
/// These warnings carry no file or line. There is no site to point at: that
/// is the whole finding. The earlier version named the rule's
/// `source_document` at line 0, which sent a reader to a policy document that
/// has nothing to say about the missing test.
fn unverified_rule_warnings(
    rules: &[provenance_core::Rule],
    scans: &[provenance_scanner::FileScan],
) -> Vec<provenance_core::coverage::ValidationWarning> {
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
        .filter(|rule| !verified.contains(rule.id.as_str()))
        .map(|rule| provenance_core::coverage::ValidationWarning {
            rule_id: rule.id.as_str().to_string(),
            file_path: None,
            line: None,
            message: format!("active rule `{}` has no verification", rule.id.as_str()),
        })
        .collect()
}

pub(super) fn current_git_commit(repo: &camino::Utf8Path) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo)
        .output()?;
    anyhow::ensure!(output.status.success(), "git rev-parse failed");
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Said of a verification site that lives in a different file from the
/// `#[rule]` attribute it checks.
const OUTSIDE_DEFINING_MODULE: &str = " (outside defining module)";

/// Where each rule is defined: the file holding its `#[rule]` site, which the
/// scanner reports as a binding with no verification method.
///
/// A rule with no `#[rule]` site in the scanned tree is absent from the map,
/// and its verification sites are then left unannotated. Nothing is known
/// about where it belongs, so nothing is claimed.
fn defining_modules(
    report: &provenance_core::coverage::CoverageReport,
) -> BTreeMap<&str, &camino::Utf8Path> {
    report
        .bindings
        .iter()
        .filter(|binding| binding.verification.is_none())
        .map(|binding| (binding.rule_id.as_str(), binding.file_path.as_path()))
        .collect()
}

/// Whether this site checks a rule defined somewhere else.
///
/// This is what a change author wants out of the report: the sites that lean
/// on the rule from outside the module that owns it, which is where a change
/// to the rule breaks somebody else's tests.
fn is_outside_defining_module(
    binding: &provenance_core::coverage::BindingResult,
    defining_modules: &BTreeMap<&str, &camino::Utf8Path>,
) -> bool {
    binding.verification.is_some()
        && defining_modules
            .get(binding.rule_id.as_str())
            .is_some_and(|defining| *defining != binding.file_path.as_path())
}

/// The rule a warning is about, ready to sit after the word `Warning`.
///
/// Empty for a warning about no rule in particular, and then the line reads
/// `- Warning in `path`:line: message`. An empty pair of backticks would only
/// look like a rule whose name went missing.
fn warning_subject(rule_id: &str) -> String {
    if rule_id.is_empty() {
        String::new()
    } else {
        format!(" `{rule_id}`")
    }
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
                annotation.rule_id, annotation.file_path, annotation.line, annotation.coverage
            )?;
        }
        let defining_modules = defining_modules(report);
        for binding in &report.bindings {
            let relation = binding.verification.as_ref().map_or_else(
                || "is the rule".to_string(),
                |method| format!("verified by {method}"),
            );
            writeln!(
                out,
                "- `{}` {} at `{}`:{}{}{}",
                binding.rule_id,
                relation,
                binding.file_path,
                binding.line,
                binding
                    .item_name
                    .as_deref()
                    .map(|name| format!(" ({name})"))
                    .unwrap_or_default(),
                if is_outside_defining_module(binding, &defining_modules) {
                    OUTSIDE_DEFINING_MODULE
                } else {
                    ""
                }
            )?;
        }
        for warning in &report.warnings {
            let subject = warning_subject(&warning.rule_id);
            match (&warning.file_path, warning.line) {
                (Some(file_path), Some(line)) => writeln!(
                    out,
                    "- Warning{subject} in `{file_path}`:{line}: {}",
                    warning.message
                )?,
                _ => writeln!(out, "- Warning{subject}: {}", warning.message)?,
            }
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
mod git_tests;

#[cfg(test)]
mod parse_warning_tests;

#[cfg(test)]
mod render_tests;

#[cfg(test)]
mod unverified_tests;
