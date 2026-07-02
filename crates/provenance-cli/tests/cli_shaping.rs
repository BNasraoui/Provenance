use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn cli_shaping_records_roundtrip_materialize_and_accept_topic_question_threads() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let import_repo = dir.path().join("imported").to_string_lossy().to_string();
    let export_path = dir
        .path()
        .join("shaping-export.json")
        .to_string_lossy()
        .to_string();
    let import_export_path = dir
        .path()
        .join("shaping-import-export.json")
        .to_string_lossy()
        .to_string();

    init(&repo);
    create_source_and_requirement(&repo);
    create_boundary(&repo);
    create_topic(&repo);
    create_question(&repo);
    post_topic_thread(&repo);
    post_question_thread(&repo);
    verify_materialized_lists(&repo);
    export_scope(&repo, &export_path);
    import_export_roundtrip(&import_repo, &export_path, &import_export_path);
}

#[test]
fn cli_shaping_turn_state_supports_claims_fog_and_answering() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();

    init(&repo);
    create_source_and_requirement(&repo);
    create_topic(&repo);
    create_question_requires_and_records_method(&repo);
    topic_claims_are_check_and_set(&repo);
    question_claims_clear_when_answered(&repo);
    closing_topic_clears_claim(&repo);
    fog_is_set_shown_and_cleared(&repo);
}

#[test]
fn cli_questions_update_changes_method_status_links_and_resolution_id() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();

    init(&repo);
    create_source_and_requirement(&repo);
    create_topic(&repo);
    create_claimed_fork_question(&repo);
    create_resolution(&repo);
    update_question_to_blocked_prototype(&repo);
    question_update_rejects_invalid_links(&repo);
    question_update_rejects_answered_without_answer(&repo);
}

fn create_claimed_fork_question(repo: &str) {
    provenance(&[
        "questions",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_fork",
        "--topic-id",
        "topic_overtime",
        "--question",
        "Which UI direction should the shaping map use?",
        "--method",
        "grill",
        "--format",
        "json",
    ])
    .success();
    provenance(&[
        "questions",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_fork",
        "--actor",
        "agent-one",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""claimed_by": "agent-one""#));
}

fn update_question_to_blocked_prototype(repo: &str) {
    let updated = provenance(&[
        "questions",
        "update",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_fork",
        "--method",
        "prototype",
        "--status",
        "blocked-on-human",
        "--links-json",
        r#"[{"target_type":"source","target_id":"source_schads"},{"target_type":"resolution","target_id":"res_overtime"}]"#,
        "--resolution-id",
        "res_overtime",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""resolution_method": "prototype""#))
    .stdout(contains(r#""status": "blocked_on_human""#))
    .stdout(contains(r#""target_type": "source""#))
    .stdout(contains(r#""target_type": "resolution""#))
    .stdout(contains(r#""resolution_id": "res_overtime""#));
    let updated_stdout = String::from_utf8(updated.get_output().stdout.clone()).unwrap();
    assert!(!updated_stdout.contains("claimed_by"));

    provenance(&[
        "questions",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_fork",
        "--actor",
        "agent-two",
    ])
    .failure()
    .stderr(contains("blocked_on_human"));
}

fn question_update_rejects_invalid_links(repo: &str) {
    provenance(&[
        "questions",
        "update",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_fork",
        "--links-json",
        r#"[{"target_type":"source","target_id":"missing_source"}]"#,
    ])
    .failure()
    .stderr(contains("linked artifact does not exist"));
}

fn question_update_rejects_answered_without_answer(repo: &str) {
    provenance(&[
        "questions",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_unanswered",
        "--topic-id",
        "topic_overtime",
        "--question",
        "What still needs a real answer?",
        "--method",
        "grill",
        "--format",
        "json",
    ])
    .success();
    provenance(&[
        "questions",
        "update",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_unanswered",
        "--status",
        "answered",
    ])
    .failure()
    .stderr(contains("use questions answer"));
}

fn create_question_requires_and_records_method(repo: &str) {
    provenance(&[
        "questions",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_no_method",
        "--topic-id",
        "topic_overtime",
        "--question",
        "Missing method?",
    ])
    .failure()
    .stderr(contains("--method"));
    provenance(&[
        "questions",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_threshold",
        "--topic-id",
        "topic_overtime",
        "--question",
        "Which threshold applies?",
        "--method",
        "research",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""resolution_method": "research""#));
}

fn topic_claims_are_check_and_set(repo: &str) {
    provenance(&[
        "topics",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "topic_overtime",
        "--actor",
        "agent-one",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""claimed_by": "agent-one""#));
    provenance(&[
        "topics",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "topic_overtime",
        "--actor",
        "agent-two",
    ])
    .failure()
    .stderr(contains(
        "topic topic_overtime is already claimed by agent-one",
    ));
    provenance(&[
        "topics",
        "release",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "topic_overtime",
        "--format",
        "json",
    ])
    .success();
}

fn question_claims_clear_when_answered(repo: &str) {
    provenance(&[
        "questions",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_threshold",
        "--actor",
        "agent-one",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""claimed_by": "agent-one""#));
    provenance(&[
        "questions",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_threshold",
        "--actor",
        "agent-two",
    ])
    .failure()
    .stderr(contains(
        "question question_threshold is already claimed by agent-one",
    ));
    let answered = provenance(&[
        "questions",
        "answer",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_threshold",
        "--answer",
        "Use the SCHADS overtime threshold.",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""status": "answered""#));
    let answered_stdout = String::from_utf8(answered.get_output().stdout.clone()).unwrap();
    assert!(!answered_stdout.contains("claimed_by"));
}

fn closing_topic_clears_claim(repo: &str) {
    provenance(&[
        "topics",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "topic_overtime",
        "--actor",
        "agent-two",
        "--format",
        "json",
    ])
    .success();
    let closed = provenance(&[
        "topics",
        "close",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "topic_overtime",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""status": "closed""#));
    let closed_stdout = String::from_utf8(closed.get_output().stdout.clone()).unwrap();
    assert!(!closed_stdout.contains("claimed_by"));
    provenance(&[
        "topics",
        "claim",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "topic_overtime",
        "--actor",
        "agent-one",
    ])
    .failure()
    .stderr(contains("closed"));
}

fn fog_is_set_shown_and_cleared(repo: &str) {
    let fog_command = |action: &'static str| {
        [
            "requirements",
            "fog",
            action,
            "--repo",
            repo,
            "--scope",
            "default",
            "--requirement-id",
            "req_overtime",
            "--format",
            "json",
        ]
    };
    let mut set_args = fog_command("set").to_vec();
    set_args.extend([
        "--text",
        "something about sleepovers; broken shifts feel relevant",
    ]);
    provenance(&set_args).success().stdout(contains(
        "something about sleepovers; broken shifts feel relevant",
    ));
    provenance(&fog_command("show"))
        .success()
        .stdout(contains(r#""requirement_id": "req_overtime""#))
        .stdout(contains(
            "something about sleepovers; broken shifts feel relevant",
        ));
    provenance(&fog_command("clear")).success();
    let shown = provenance(&fog_command("show")).success();
    let shown_stdout = String::from_utf8(shown.get_output().stdout.clone()).unwrap();
    assert!(shown_stdout.contains(r#""fog": null"#));
}

#[test]
fn cli_questions_create_help_includes_sizing_guidance() {
    provenance(&["questions", "create", "--help"])
        .success()
        .stdout(contains(
            "A question should be resolvable in one agent session",
        ))
        .stdout(contains("otherwise it is fog or needs decomposition"));
}

fn provenance(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("provenance")
        .unwrap()
        .args(args)
        .assert()
}

fn init(repo: &str) {
    provenance(&[
        "init",
        "--path",
        repo,
        "--scope",
        "default",
        "--path-prefix",
        ".",
    ])
    .success();
}

fn create_source_and_requirement(repo: &str) {
    provenance(&[
        "sources",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "source_schads",
        "--name",
        "SCHADS Award",
        "--format",
        "json",
    ])
    .success();
    provenance(&[
        "requirements",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "req_overtime",
        "--statement",
        "Overtime must follow SCHADS thresholds",
        "--format",
        "json",
    ])
    .success();
}

fn create_boundary(repo: &str) {
    provenance(&[
        "boundaries",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "boundary_no_manual_rework",
        "--requirement-id",
        "req_overtime",
        "--statement",
        "No manual payroll reconciliation",
        "--source-id",
        "source_schads",
        "--source-clause",
        "28.1",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("boundary_no_manual_rework"))
    .stdout(contains(r#""source_id": "source_schads""#));
}

fn create_topic(repo: &str) {
    provenance(&[
        "topics",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "topic_overtime",
        "--requirement-id",
        "req_overtime",
        "--title",
        "Overtime eligibility",
        "--status",
        "open",
        "--links-json",
        r#"[{"target_type":"source","target_id":"source_schads"}]"#,
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("topic_overtime"))
    .stdout(contains(r#""status": "open""#));
}

fn create_question(repo: &str) {
    provenance(&[
        "questions",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "question_overtime_threshold",
        "--topic-id",
        "topic_overtime",
        "--question",
        "Which threshold applies?",
        "--method",
        "grill",
        "--status",
        "answered",
        "--answer",
        "Use the SCHADS overtime threshold.",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("question_overtime_threshold"))
    .stdout(contains(r#""resolution_method": "grill""#))
    .stdout(contains(r#""requirement_id": "req_overtime""#));
}

fn create_resolution(repo: &str) {
    provenance(&[
        "resolutions",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "res_overtime",
        "--title",
        "Overtime threshold",
        "--requirement-id",
        "req_overtime",
        "--position",
        "Use the SCHADS threshold.",
        "--rationale",
        "Human confirmed the source threshold.",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("res_overtime"));
}

fn post_topic_thread(repo: &str) {
    provenance(&[
        "thread",
        "post",
        "--repo",
        repo,
        "--scope",
        "default",
        "--parent-type",
        "topic",
        "--parent-id",
        "topic_overtime",
        "--role",
        "assistant",
        "Work this topic before shaping the pitch",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("thread_topic_topic_overtime"));
}

fn post_question_thread(repo: &str) {
    provenance(&[
        "thread",
        "post",
        "--repo",
        repo,
        "--scope",
        "default",
        "--parent-type",
        "question",
        "--parent-id",
        "question_overtime_threshold",
        "--role",
        "assistant",
        "Answered from SCHADS clause 28.1",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("thread_question_question_overtime_threshold"));
}

fn verify_materialized_lists(repo: &str) {
    provenance(&["materialize", "--repo", repo, "--format", "json"])
        .success()
        .stdout(contains(r#""records_loaded""#));
    provenance(&[
        "topics", "list", "--repo", repo, "--scope", "default", "--format", "json",
    ])
    .success()
    .stdout(contains("topic_overtime"));
    provenance(&[
        "questions",
        "list",
        "--repo",
        repo,
        "--scope",
        "default",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("question_overtime_threshold"));
    provenance(&[
        "boundaries",
        "list",
        "--repo",
        repo,
        "--scope",
        "default",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("boundary_no_manual_rework"));
}

fn export_scope(repo: &str, export_path: &str) {
    provenance(&[
        "export",
        "--repo",
        repo,
        "--scope",
        "default",
        "--format",
        "json",
        "--output",
        export_path,
    ])
    .success();
}

fn import_export_roundtrip(import_repo: &str, export_path: &str, import_export_path: &str) {
    init(import_repo);
    provenance(&[
        "import",
        "--repo",
        import_repo,
        "--scope",
        "default",
        "--input",
        export_path,
        "--format",
        "json",
    ])
    .success();
    export_scope(import_repo, import_export_path);

    let exported = std::fs::read_to_string(import_export_path).unwrap();
    assert!(exported.contains(r#""boundaries""#));
    assert!(exported.contains(r#""topics""#));
    assert!(exported.contains(r#""questions""#));
    assert!(exported.contains("boundary_no_manual_rework"));
    assert!(exported.contains("topic_overtime"));
    assert!(exported.contains("question_overtime_threshold"));
}
