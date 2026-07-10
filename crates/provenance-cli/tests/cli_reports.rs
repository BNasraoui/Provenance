use assert_cmd::Command;
use std::path::Path;

#[allow(clippy::too_many_lines)]
fn seed_repo() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            &repo,
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
            "sources",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "source_schads",
            "--name",
            "SCHADS Award",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "domains",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "domain_payroll",
            "--name",
            "Payroll",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "requirements",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "req_schads_overtime",
            "--statement",
            "Overtime must follow SCHADS thresholds",
            "--domain-id",
            "domain_payroll",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "requirements",
            "source-ref",
            "add",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--requirement-id",
            "req_schads_overtime",
            "--source-id",
            "source_schads",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "resolutions",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "res_schads_overtime",
            "--title",
            "Overtime interpretation",
            "--requirement-id",
            "req_schads_overtime",
            "--position",
            "Use award threshold",
            "--rationale",
            "Matches source clause",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "rules",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "rule_schads_pay_001",
            "--rule-code",
            "SCHADS-PAY-001",
            "--requirement-id",
            "req_schads_overtime",
            "--resolution-id",
            "res_schads_overtime",
            "--statement",
            "Pay overtime after the threshold",
            "--severity",
            "high",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args(["materialize", "--repo", &repo, "--format", "json"])
        .assert()
        .success();
    (dir, repo)
}

fn mark_resolution_stale(repo: &str) {
    let path = Path::new(repo).join(".provenance/state/scopes/default/resolutions/res.jsonl");
    let contents = std::fs::read_to_string(&path).unwrap();
    let mut resolution: serde_json::Value = serde_json::from_str(contents.trim()).unwrap();
    resolution["status"] = serde_json::json!("approved");
    resolution["approved_at"] = serde_json::json!(0);
    resolution["review_on"] = serde_json::json!("2000-01-01");
    std::fs::write(
        path,
        format!("{}\n", serde_json::to_string(&resolution).unwrap()),
    )
    .unwrap();
}

fn stale_resolution_ids(repo: &str, filters: &[&str]) -> Vec<String> {
    let mut args = vec![
        "stale", "--repo", repo, "--scope", "default", "--format", "json",
    ];
    args.extend_from_slice(filters);
    let output = Command::cargo_bin("provenance")
        .unwrap()
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Vec<serde_json::Value>>(&output.stdout)
        .unwrap()
        .into_iter()
        .map(|item| item["resolution_id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn cli_reports_emit_json_shapes_for_impact_stale_health_and_orphans() {
    let (_dir, repo) = seed_repo();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "impact",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--node-type",
            "source",
            "source_schads",
            "--max-hops",
            "3",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("rule_schads_pay_001"))
        .stdout(predicates::str::contains("downstream"));
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "stale", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("[]"));
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "health", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("with_complete_traceability"))
        .stdout(predicates::str::contains("source_linked_requirements"));
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "orphans", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("[]"));
}

#[test]
fn stale_filters_by_age_severity_and_downstream_rule_count() {
    let (_dir, repo) = seed_repo();
    mark_resolution_stale(&repo);

    assert_eq!(stale_resolution_ids(&repo, &[]), ["res_schads_overtime"]);
    assert_eq!(
        stale_resolution_ids(&repo, &["--min-age-days", "1"]),
        ["res_schads_overtime"]
    );
    assert!(stale_resolution_ids(&repo, &["--min-age-days", "4294967295"]).is_empty());
    assert_eq!(
        stale_resolution_ids(&repo, &["--rule-severities", "medium,high"]),
        ["res_schads_overtime"]
    );
    assert!(stale_resolution_ids(&repo, &["--rule-severities", "critical"]).is_empty());
    assert_eq!(
        stale_resolution_ids(&repo, &["--min-downstream-rules", "1"]),
        ["res_schads_overtime"]
    );
    assert!(stale_resolution_ids(&repo, &["--min-downstream-rules", "2"]).is_empty());
}
