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
    let known_rules = if validate_rules {
        StateStore::new(ProvenanceLayout::new(repo))
            .list_rules(&ScopeId::new(scope)?)?
            .into_iter()
            .map(|rule| rule.rule_code)
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let scanner_warnings = if validate_rules {
        provenance_scanner::validate_annotations(&scans, known_rules.iter().cloned())
    } else {
        Vec::new()
    };
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
    Ok(provenance_core::coverage::CoverageReport::new(
        current_git_commit().ok(),
        scans.len(),
        annotations,
        warnings,
    ))
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
        }
    }
    Ok(())
}
