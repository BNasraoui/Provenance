#[path = "shaping_support/fixtures.rs"]
mod fixtures;
#[path = "shaping_support/provenance.rs"]
mod provenance;

use fixtures::{create_source_and_requirement, create_topic, init};
use predicates::str::contains;
use provenance::provenance;

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
