use super::*;

#[test]
fn explicit_commit_issues_from_pin_despite_relevant_staged_and_worktree_changes() {
    let temp = committed_store();
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let head = String::from_utf8(head.stdout).unwrap().trim().to_string();
    let manifest = temp.path().join(".provenance/state/manifest.json");
    let original = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(&manifest, original.replace("\".\"", "\"staged\"")).unwrap();
    git(temp.path(), &["add", ".provenance/state/manifest.json"]);
    std::fs::write(&manifest, original.replace("\".\"", "\"worktree\"")).unwrap();

    let reference = issue(temp.path(), &["--commit", &head]);
    assert_eq!(reference["commit"], head);
}

#[test]
fn exact_export_contains_only_canonical_graph_families() {
    let temp = committed_store();
    let proposal_dir = temp
        .path()
        .join(".provenance/state/scopes/default/ideation");
    std::fs::create_dir_all(&proposal_dir).unwrap();
    std::fs::write(
        proposal_dir.join("proposal_cards.jsonl"),
        concat!(
            "{\"schema_version\":1,\"scope_id\":\"default\",",
            "\"id\":\"proposal_workflowd_123\",\"proposal_key\":\"workflowd-123\",",
            "\"proposal_type\":\"no_action\",\"title\":\"No graph change\",",
            "\"summary\":\"Collaboration-only proposal\",\"traceability\":{",
            "\"target\":{\"artifact_type\":\"requirement\",\"artifact_id\":\"req_none\"},",
            "\"source_ids\":[],\"evidence_references\":[],\"supporting_claim_ids\":[]},",
            "\"promotion_state\":\"proposed\"}\n"
        ),
    )
    .unwrap();
    git(temp.path(), &["add", ".provenance/state"]);
    git(
        temp.path(),
        &["commit", "-qm", "non-graph collaboration state"],
    );
    let reference = issue(temp.path(), &[]);
    let reference_path = write_reference(temp.path(), &reference);

    let output = provenance(temp.path())
        .args([
            "graph-reference",
            "exact-export",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export: Value = serde_json::from_slice(&output).unwrap();
    let graph = export["graph"].as_object().unwrap();
    assert!(!graph.contains_key("proposal_cards"));
    assert!(!graph.contains_key("promotion_decisions"));
    assert!(!String::from_utf8(output.clone())
        .unwrap()
        .contains("proposal_workflowd_123"));
    assert!(!String::from_utf8(output).unwrap().contains("workflowd-123"));
}

#[test]
fn collaboration_claims_do_not_change_digest_or_appear_in_exact_export() {
    let temp = committed_store();
    provenance(temp.path())
        .args([
            "sources",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "source_origin",
            "--name",
            "Origin metadata source",
            "--origin-thread",
            "thread_private",
            "--origin-message",
            "message_private",
        ])
        .assert()
        .success();
    provenance(temp.path())
        .args([
            "requirements",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "req_claims",
            "--statement",
            "Claims are collaboration metadata",
        ])
        .assert()
        .success();
    provenance(temp.path())
        .args([
            "topics",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "topic_claims",
            "--requirement-id",
            "req_claims",
            "--title",
            "Claim handling",
        ])
        .assert()
        .success();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "add graph topic"]);
    let unclaimed = issue(temp.path(), &[]);

    provenance(temp.path())
        .args([
            "topics",
            "claim",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "topic_claims",
            "--actor",
            "workflowd-123",
        ])
        .assert()
        .success();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "claim graph topic"]);
    let claimed = issue(temp.path(), &[]);

    assert_eq!(unclaimed["graph_digest"], claimed["graph_digest"]);
    let reference_path = write_reference(temp.path(), &claimed);
    let output = provenance(temp.path())
        .args([
            "graph-reference",
            "exact-export",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("claimed_by"));
    assert!(!output.contains("claimed_at"));
    assert!(!output.contains("workflowd-123"));
    assert!(!output.contains("origin_thread"));
    assert!(!output.contains("origin_message"));
    assert!(!output.contains("thread_private"));
    assert!(!output.contains("message_private"));
}
