use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command as ProcessCommand;

fn git(repo: &Path, args: &[&str]) -> String {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn write(repo: &Path, relative: &str, contents: &str) {
    let path = repo.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn commit(repo: &Path, message: &str) -> String {
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", message]);
    git(repo, &["rev-parse", "HEAD"])
}

fn init_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            dir.path().to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();
    dir
}

fn seed_graph(repo: &Path, base: &str) {
    let scope = repo.join(".provenance/state/scopes/default");
    write(
        repo,
        ".provenance/state/scopes/default/sources/source.jsonl",
        &format!(
            "{{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"source_code\",\"name\":\"code\",\"source_type\":\"system_state\",\"commit_pin\":\"{base}\"}}\n"
        ),
    );
    write(
        repo,
        ".provenance/state/scopes/default/requirements/req.jsonl",
        "{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"req_ratified\",\"statement\":\"The guard must reject invalid input\",\"status\":\"active\"}\n",
    );
    write(
        repo,
        ".provenance/state/scopes/default/rules/rule.jsonl",
        "{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"rule_guard\",\"rule_code\":\"GUARD-1\",\"statement\":\"Reject invalid input\",\"status\":\"active\",\"severity\":\"high\"}\n",
    );
    write(
        repo,
        ".provenance/state/edges/edges-00.jsonl",
        "{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"edge_produces\",\"edge_type\":\"produces\",\"from_type\":\"requirement\",\"from_id\":\"req_ratified\",\"to_type\":\"rule\",\"to_id\":\"rule_guard\"}\n",
    );
    let proposals = r#"{"schema_version":1,"scope_id":"default","id":"proposal_ratified","proposal_key":"backtrace/guard","proposal_type":"requirement_candidate","title":"The guard must reject invalid input","summary":"ratified","traceability":{"target":{"artifact_type":"requirement","artifact_id":"req_ratified"},"source_ids":["source_code"],"evidence_references":[{"reference_id":"ev_moved","evidence_type":"artifact","summary":"guard implementation","file_path":"src/guard.rs","line":1},{"reference_id":"ev_vanished","evidence_type":"artifact","summary":"rejection branch","file_path":"src/reject.rs","line":1}],"supporting_claim_ids":[]},"promotion_state":"accepted"}
{"schema_version":1,"scope_id":"default","id":"proposal_unaffected","proposal_key":"backtrace/other","proposal_type":"requirement_candidate","title":"Other behavior","summary":"proposed","traceability":{"target":{"artifact_type":"source","artifact_id":"source_code"},"source_ids":["source_code"],"evidence_references":[{"reference_id":"ev_unchanged","evidence_type":"artifact","summary":"other implementation","file_path":"src/other.rs","line":1}],"supporting_claim_ids":[]},"promotion_state":"proposed"}
"#;
    fs::create_dir_all(scope.join("ideation")).unwrap();
    fs::write(scope.join("ideation/proposal_cards.jsonl"), proposals).unwrap();
}

fn evidence_review_json(repo: &Path, extra: &[&str]) -> Value {
    let mut command = Command::cargo_bin("provenance").unwrap();
    command.args([
        "evidence-review",
        "--repo",
        repo.to_str().unwrap(),
        "--scope",
        "default",
        "--format",
        "json",
    ]);
    command.args(extra);
    let output = command.assert().success().get_output().stdout.clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn evidence_review_reverifies_only_intersecting_evidence() {
    let dir = init_repo();
    write(dir.path(), "src/guard.rs", "pub fn guard() {}\n");
    write(dir.path(), "src/reject.rs", "return Err(\"invalid\");\n");
    write(dir.path(), "src/other.rs", "pub fn other() {}\n");
    let base = commit(dir.path(), "base");
    seed_graph(dir.path(), &base);
    fs::rename(
        dir.path().join("src/guard.rs"),
        dir.path().join("src/renamed_guard.rs"),
    )
    .unwrap();
    write(dir.path(), "src/renamed_guard.rs", "pub fn guard() {}\n");
    write(dir.path(), "src/reject.rs", "return Ok(());\n");
    write(dir.path(), "notes.txt", "unrelated\n");
    commit(dir.path(), "change evidence");

    let report = evidence_review_json(dir.path(), &[]);
    assert_eq!(report["summary"]["graph_evidence_paths"], 3);
    assert_eq!(report["summary"]["intersecting_paths"], 2);
    assert_eq!(report["summary"]["evidence_reverified"], 2);
    assert_eq!(report["evidence"][0]["status"], "moved");
    assert_eq!(report["evidence"][0]["current_line"], 1);
    assert_eq!(
        report["evidence"][0]["current_path"],
        "src/renamed_guard.rs"
    );
    assert_eq!(report["evidence"][1]["status"], "vanished");
    assert_eq!(
        report["contradictions"][0]["requirement_id"],
        "req_ratified"
    );
    assert_eq!(
        report["contradictions"][0]["evidence_reference_id"],
        "ev_vanished"
    );
    assert!(!report.to_string().contains("ev_unchanged"));
}

#[test]
fn evidence_review_honors_downstream_rule_and_severity_filters() {
    let dir = init_repo();
    write(dir.path(), "src/reject.rs", "return Err(\"invalid\");\n");
    let base = commit(dir.path(), "base");
    seed_graph(dir.path(), &base);
    write(dir.path(), "src/reject.rs", "return Ok(());\n");
    commit(dir.path(), "contradict requirement");

    let severity = evidence_review_json(dir.path(), &["--rule-severities", "critical"]);
    assert_eq!(severity["summary"]["evidence_reverified"], 0);
    assert_eq!(severity["contradictions"], serde_json::json!([]));

    let downstream = evidence_review_json(dir.path(), &["--min-downstream-rules", "2"]);
    assert_eq!(downstream["summary"]["evidence_reverified"], 0);
    assert_eq!(downstream["contradictions"], serde_json::json!([]));
}

#[test]
fn evidence_review_base_override_supports_ci_diff_ranges() {
    let dir = init_repo();
    write(dir.path(), "src/reject.rs", "return Err(\"invalid\");\n");
    let base = commit(dir.path(), "base");
    seed_graph(dir.path(), &base);
    write(dir.path(), "src/reject.rs", "return Ok(());\n");
    let head = commit(dir.path(), "change");

    let report = evidence_review_json(dir.path(), &["--base", &base, "--head", &head]);
    assert_eq!(report["diffs"][0]["base"], base);
    assert_eq!(report["diffs"][0]["head"], head);
    assert_eq!(report["evidence"][0]["status"], "vanished");
}

#[test]
fn stale_compares_review_dates_with_today_instead_of_2099() {
    let dir = init_repo();
    write(
        dir.path(),
        ".provenance/state/scopes/default/resolutions/res.jsonl",
        r#"{"schema_version":1,"scope_id":"default","id":"res_past","title":"Past","position":"x","rationale":"x","status":"approved","inputs":[],"review_on":"2020-01-01","review_triggers":[]}
{"schema_version":1,"scope_id":"default","id":"res_future","title":"Future","position":"x","rationale":"x","status":"approved","inputs":[],"review_on":"2099-01-01","review_triggers":[]}
"#,
    );

    let output = Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "stale",
            "--repo",
            dir.path().to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report.as_array().unwrap().len(), 1);
    assert_eq!(report[0]["resolution_id"], "res_past");
}

#[test]
fn evidence_review_rejects_ambiguous_multi_source_sites() {
    let dir = init_repo();
    write(dir.path(), "src/reject.rs", "return Err(\"invalid\");\n");
    let base = commit(dir.path(), "base");
    seed_graph(dir.path(), &base);
    let sources = dir
        .path()
        .join(".provenance/state/scopes/default/sources/source.jsonl");
    fs::OpenOptions::new()
        .append(true)
        .open(sources)
        .unwrap()
        .write_all(format!("{{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"source_other\",\"name\":\"other\",\"source_type\":\"system_state\",\"commit_pin\":\"{base}\"}}\n").as_bytes())
        .unwrap();
    let proposals = dir
        .path()
        .join(".provenance/state/scopes/default/ideation/proposal_cards.jsonl");
    let contents = fs::read_to_string(&proposals).unwrap().replacen(
        "\"source_ids\":[\"source_code\"]",
        "\"source_ids\":[\"source_code\",\"source_other\"]",
        1,
    );
    fs::write(proposals, contents).unwrap();
    write(dir.path(), "src/reject.rs", "return Ok(());\n");
    commit(dir.path(), "change");

    let report = evidence_review_json(dir.path(), &[]);
    assert_eq!(report["summary"]["evidence_reverified"], 0);
    assert!(report["diagnostics"][0]
        .as_str()
        .unwrap()
        .contains("ambiguous sources"));
}

#[test]
fn accepted_proposal_id_is_not_fabricated_as_requirement_id() {
    let dir = init_repo();
    write(dir.path(), "src/reject.rs", "return Err(\"invalid\");\n");
    let base = commit(dir.path(), "base");
    seed_graph(dir.path(), &base);
    let proposals = dir
        .path()
        .join(".provenance/state/scopes/default/ideation/proposal_cards.jsonl");
    let only_source_target = fs::read_to_string(&proposals)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .replace(
            "\"artifact_type\":\"requirement\",\"artifact_id\":\"req_ratified\"",
            "\"artifact_type\":\"source\",\"artifact_id\":\"source_code\"",
        );
    fs::write(proposals, format!("{only_source_target}\n")).unwrap();
    write(dir.path(), "src/reject.rs", "return Ok(());\n");
    commit(dir.path(), "change");

    let report = evidence_review_json(dir.path(), &[]);
    assert_eq!(report["evidence"][0]["status"], "vanished");
    assert_eq!(report["contradictions"], serde_json::json!([]));
}

#[test]
fn evidence_review_uses_rules_produced_by_resolving_resolutions() {
    let dir = init_repo();
    write(dir.path(), "src/reject.rs", "return Err(\"invalid\");\n");
    let base = commit(dir.path(), "base");
    seed_graph(dir.path(), &base);
    write(
        dir.path(),
        ".provenance/state/scopes/default/resolutions/res.jsonl",
        "{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"res_guard\",\"title\":\"Guard\",\"position\":\"reject\",\"rationale\":\"safe\",\"status\":\"approved\",\"inputs\":[],\"review_triggers\":[]}\n",
    );
    write(
        dir.path(),
        ".provenance/state/edges/edges-00.jsonl",
        "{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"edge_resolves\",\"edge_type\":\"resolves\",\"from_type\":\"resolution\",\"from_id\":\"res_guard\",\"to_type\":\"requirement\",\"to_id\":\"req_ratified\"}\n{\"schema_version\":1,\"scope_id\":\"default\",\"id\":\"edge_produces\",\"edge_type\":\"produces\",\"from_type\":\"resolution\",\"from_id\":\"res_guard\",\"to_type\":\"rule\",\"to_id\":\"rule_guard\"}\n",
    );
    write(dir.path(), "src/reject.rs", "return Ok(());\n");
    commit(dir.path(), "change");

    let report = evidence_review_json(
        dir.path(),
        &["--rule-severities", "high", "--min-downstream-rules", "1"],
    );
    assert_eq!(report["summary"]["evidence_reverified"], 1);
    assert_eq!(report["evidence"][0]["status"], "vanished");
}

#[test]
fn evidence_review_rejects_ambiguous_canonical_requirement_ownership() {
    let dir = init_repo();
    write(dir.path(), "src/reject.rs", "return Err(\"invalid\");\n");
    let base = commit(dir.path(), "base");
    seed_graph(dir.path(), &base);
    write(
        dir.path(),
        ".provenance/state/scopes/default/ideation/promotion_decisions.jsonl",
        r#"{"schema_version":1,"scope_id":"default","id":"decision_one","proposal_id":"proposal_ratified","decision":"accepted","rationale":"one","actor":{"identity_type":"human","id":"one"},"canonical_artifact":{"artifact_type":"requirement","artifact_id":"req_one"}}
{"schema_version":1,"scope_id":"default","id":"decision_two","proposal_id":"proposal_ratified","decision":"accepted","rationale":"two","actor":{"identity_type":"human","id":"two"},"canonical_artifact":{"artifact_type":"requirement","artifact_id":"req_two"}}
"#,
    );
    write(dir.path(), "src/reject.rs", "return Ok(());\n");
    commit(dir.path(), "change");

    let report = evidence_review_json(dir.path(), &[]);
    assert_eq!(report["summary"]["evidence_reverified"], 0);
    assert!(report["diagnostics"][0]
        .as_str()
        .unwrap()
        .contains("ambiguous canonical requirements"));
}
