#[path = "shaping_support/fixtures.rs"]
mod fixtures;
#[path = "shaping_support/provenance.rs"]
mod provenance;

use fixtures::{create_source_and_requirement, create_topic, init};
use predicates::str::contains;
use provenance::provenance;

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
