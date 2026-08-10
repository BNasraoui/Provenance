use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn coverage_scan_reports_unknown_rule_warnings() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    let source_dir = repo.join("src");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(
        source_dir.join("payroll.rs"),
        "// @provenance rule: UNKNOWN-RULE\nfn pays_overtime() {}\n",
    )
    .unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "coverage",
            "scan",
            "--repo",
            repo.to_str().unwrap(),
            "--path",
            source_dir.to_str().unwrap(),
            "--scope",
            "default",
            "--validate-rules",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("UNKNOWN-RULE"))
        .stdout(predicate::str::contains("total_annotations"));
}

#[test]
fn coverage_scan_writes_markdown_output_file() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    let source_dir = repo.join("src");
    let output = repo.join("coverage.md");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(
        source_dir.join("payroll.py"),
        "# @provenance rule: SCHADS-PAY-001\ndef pays_overtime():\n    pass\n",
    )
    .unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "coverage",
            "scan",
            "--repo",
            repo.to_str().unwrap(),
            "--path",
            source_dir.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "markdown",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let markdown = std::fs::read_to_string(output).unwrap();
    assert!(markdown.contains("# Coverage Scan"));
    assert!(markdown.contains("SCHADS-PAY-001"));
}

#[test]
fn coverage_scan_strict_exits_non_zero_on_unverified_active_rule_then_zero_once_verified() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    let source_dir = repo.join("src");
    std::fs::create_dir_all(&source_dir).unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "rules",
            "create",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--id",
            "rule_pays_overtime",
            "--rule-code",
            "SCHADS-PAY-001",
            "--statement",
            "Pay overtime after the threshold",
            "--severity",
            "high",
        ])
        .assert()
        .success();

    std::fs::write(source_dir.join("payroll.rs"), "fn pays_overtime() {}\n").unwrap();

    // Unverified: --strict fails, report still printed.
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "coverage",
            "scan",
            "--repo",
            repo.to_str().unwrap(),
            "--path",
            source_dir.to_str().unwrap(),
            "--scope",
            "default",
            "--validate-rules",
            "--strict",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("has no verification"));

    // Verified via a comment annotation carrying a `verification:` key.
    std::fs::write(
        source_dir.join("payroll.rs"),
        "// @provenance rule: SCHADS-PAY-001\n// @provenance verification: examples\nfn pays_overtime() {}\n",
    )
    .unwrap();

    // Verified: --strict succeeds.
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "coverage",
            "scan",
            "--repo",
            repo.to_str().unwrap(),
            "--path",
            source_dir.to_str().unwrap(),
            "--scope",
            "default",
            "--validate-rules",
            "--strict",
            "--format",
            "json",
        ])
        .assert()
        .success();
}
