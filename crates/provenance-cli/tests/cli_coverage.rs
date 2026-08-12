use assert_cmd::Command;
use predicates::prelude::*;

fn strict_validating_scan(repo: &std::path::Path, source_dir: &std::path::Path) -> Command {
    let mut command = Command::cargo_bin("provenance").unwrap();
    command.args([
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
    ]);
    command
}

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

/// A change author reading the report wants to know who leans on an
/// implementation from another module, because that is whose tests a change
/// to the implementation breaks.
#[test]
fn coverage_markdown_marks_verification_sites_outside_the_implementation_module() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    let source_dir = repo.join("src");
    let output = repo.join("coverage.md");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(
        source_dir.join("payroll.rs"),
        "#[rule(\"rule_pays_overtime\")]\npub fn pays_overtime() {}\n\n#[verifies(\"rule_pays_overtime\", exhaustion)]\nfn covers_every_threshold() {}\n",
    )
    .unwrap();
    std::fs::write(
        source_dir.join("billing.rs"),
        "#[verifies(\"rule_pays_overtime\", examples)]\nfn bills_overtime_at_the_right_rate() {}\n",
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
    assert!(
        markdown.contains("verified by examples at `")
            && markdown.contains(
                "billing.rs`:1 (bills_overtime_at_the_right_rate) (new) (outside implementation module)"
            ),
        "billing site not marked as outside the implementation module:\n{markdown}"
    );
    assert!(
        !markdown.contains("covers_every_threshold) (outside implementation module)"),
        "site beside the rule wrongly marked:\n{markdown}"
    );
}

#[test]
fn strict_scan_reports_unverified_and_unimplemented_as_independent_findings() {
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
            "--statement",
            "Pay overtime after the threshold",
            "--severity",
            "high",
        ])
        .assert()
        .success();

    std::fs::write(source_dir.join("payroll.rs"), "fn pays_overtime() {}\n").unwrap();

    // Unverified: --strict fails, report still printed.
    strict_validating_scan(repo, repo)
        .assert()
        .failure()
        .stdout(predicate::str::contains("has no verification"));

    // Verification does not imply implementation. The old dangling-verifies
    // warning is gone because the canonical Rule exists.
    std::fs::write(
        source_dir.join("payroll.rs"),
        "#[verifies(\"rule_pays_overtime\", examples)]\nfn verifies_pays_overtime() {}\n",
    )
    .unwrap();

    strict_validating_scan(repo, repo)
        .assert()
        .failure()
        .stdout(predicate::str::contains("has no implementation"))
        .stdout(predicate::str::contains("has no #[rule]").not())
        .stdout(predicate::str::contains("has no verification").not());

    std::fs::write(
        source_dir.join("payroll.rs"),
        "#[rule(\"rule_pays_overtime\")]\nfn pays_overtime() {}\n\n#[verifies(\"rule_pays_overtime\", examples)]\nfn verifies_pays_overtime() {}\n",
    )
    .unwrap();

    strict_validating_scan(repo, repo).assert().success();
}

#[test]
fn partial_scan_does_not_claim_scope_wide_binding_absence() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    let selected = repo.join("selected");
    std::fs::create_dir_all(&selected).unwrap();
    std::fs::write(selected.join("unrelated.rs"), "fn unrelated() {}\n").unwrap();

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
            "rule_outside_selected_path",
            "--statement",
            "A Rule outside the selected scan territory",
        ])
        .assert()
        .success();

    strict_validating_scan(repo, &selected)
        .assert()
        .success()
        .stdout(predicate::str::contains("has no implementation").not())
        .stdout(predicate::str::contains("has no verification").not());
}
