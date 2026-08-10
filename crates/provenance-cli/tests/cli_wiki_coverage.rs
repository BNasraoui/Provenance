use assert_cmd::Command;

#[test]
fn wiki_build_uses_coverage_report_for_rule_functions_and_verifications() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    let out = dir.path().join("site");
    let report = dir.path().join("coverage.json");
    seed_rules(dir.path(), &repo);
    seed_git_remote(&repo);
    std::fs::write(
        &report,
        r#"{
  "commit": "abc1234",
  "files_scanned": 2,
  "total_annotations": 0,
  "warnings": [],
  "annotations": [],
  "bindings": [{
    "rule_id": "rule_bound",
    "file_path": "src/rules.rs",
    "line": 7,
    "item_name": "decide_bound_rule",
    "verification": null
  }, {
    "rule_id": "rule_bound",
    "file_path": "src/rules.rs",
    "line": 31,
    "item_name": "bound_rule_exhaustion",
    "verification": "exhaustion"
  }, {
    "rule_id": "rule_bound",
    "file_path": "tests/rules.rs",
    "line": 12,
    "item_name": "bound_rule_examples",
    "verification": "examples"
  }]
}"#,
    )
    .unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "wiki",
            "build",
            "--repo",
            &repo.to_string_lossy(),
            "--out",
            &out.to_string_lossy(),
            "--coverage",
            &report.to_string_lossy(),
        ])
        .assert()
        .success();

    let bound = std::fs::read_to_string(out.join("rules/rule_bound/index.html")).unwrap();
    assert!(bound.contains("Rule Function"), "{bound}");
    assert!(bound.contains("decide_bound_rule"), "{bound}");
    assert!(bound.contains("src/rules.rs:7"), "{bound}");
    assert!(
        bound.contains("https://github.com/example/provenance/blob/abc1234/src/rules.rs#L7"),
        "{bound}"
    );
    assert!(bound.contains("Verification"), "{bound}");
    assert!(bound.contains("exhaustion"), "{bound}");
    assert!(bound.contains("bound_rule_exhaustion"), "{bound}");
    assert!(bound.contains("examples"), "{bound}");
    assert!(bound.contains("bound_rule_examples"), "{bound}");
    assert!(bound.contains("tests/rules.rs:12"), "{bound}");
    assert_eq!(
        bound.matches("outside defining module").count(),
        1,
        "{bound}"
    );
    assert!(!bound.contains("docs/obsolete.md"), "{bound}");
    assert!(!bound.contains(">Evidence</h2>"), "{bound}");

    assert!(
        bound.contains("Code scan at commit <code>abc1234</code>"),
        "{bound}"
    );

    let unbound = std::fs::read_to_string(out.join("rules/rule_unbound/index.html")).unwrap();
    assert!(unbound.contains("No function bound"), "{unbound}");
    assert!(unbound.contains("Not verified"), "{unbound}");
    assert!(
        unbound.contains("Code scan at commit <code>abc1234</code>"),
        "{unbound}"
    );
}

fn seed_rules(dir: &std::path::Path, repo: &std::path::Path) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            &repo.to_string_lossy(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();
    let state = dir.join("state.json");
    std::fs::write(
        &state,
        r#"{
  "scope": "default",
  "sources": [],
  "requirements": [],
  "resolutions": [],
  "rules": [{
    "schema_version": 1,
    "scope_id": "default",
    "id": "rule_bound",
    "name": "Bound rule",
    "statement": "The bound decision is canonical.",
    "status": "active",
    "severity": "high",
    "source_document": "docs/obsolete.md",
    "source_section": "old_description"
  }, {
    "schema_version": 1,
    "scope_id": "default",
    "id": "rule_unbound",
    "name": "Unbound rule",
    "statement": "An absent binding is shown honestly.",
    "status": "active",
    "severity": "medium"
  }],
  "edges": [],
  "threads": [],
  "messages": []
}"#,
    )
    .unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            &repo.to_string_lossy(),
            "--scope",
            "default",
            "--input",
            &state.to_string_lossy(),
            "--format",
            "json",
        ])
        .assert()
        .success();
}

fn seed_git_remote(repo: &std::path::Path) {
    std::process::Command::new("git")
        .args(["init", repo.to_str().unwrap()])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "remote",
            "add",
            "origin",
            "git@github.com:example/provenance.git",
        ])
        .output()
        .unwrap();
}
