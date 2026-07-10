#[path = "shaping_support/fixtures.rs"]
mod fixtures;
#[path = "shaping_support/provenance.rs"]
mod provenance;

use fixtures::{create_source_and_requirement, create_topic, init};
use predicates::str::contains;
use provenance::provenance;

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
