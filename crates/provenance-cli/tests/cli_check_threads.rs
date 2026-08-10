use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::path::Path;

#[test]
fn check_rejects_thread_in_unknown_scope_and_names_only_the_offender() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path());
    let state = dir.path().join(".provenance/state");
    write_jsonl(
        &state.join("scopes/default/requirements/req.jsonl"),
        &requirement("req_parent"),
    );
    write_jsonl(
        &state.join("scopes/default/threads/threads.jsonl"),
        &format!(
            "{}\n{}",
            thread("thread_innocent", "default", "req_parent", 1),
            thread("thread_bad_scope", "missing", "req_parent", 2)
        ),
    );

    check(dir.path()).failure().stderr(
        contains("thread thread_bad_scope is in unknown scope missing")
            .and(contains("thread_innocent").not()),
    );
}

#[test]
fn check_rejects_thread_with_unknown_parent_and_names_only_the_offender() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path());
    let state = dir.path().join(".provenance/state");
    write_jsonl(
        &state.join("scopes/default/requirements/req.jsonl"),
        &requirement("req_parent"),
    );
    write_jsonl(
        &state.join("scopes/default/threads/threads.jsonl"),
        &format!(
            "{}\n{}",
            thread("thread_innocent", "default", "req_parent", 1),
            thread("thread_bad_parent", "default", "req_missing", 2)
        ),
    );

    check(dir.path()).failure().stderr(
        contains("thread thread_bad_parent has dangling reference: parent requirement req_missing")
            .and(contains("thread_innocent").not()),
    );
}

#[test]
fn check_accepts_threads_with_known_scopes_and_parents() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path());
    let state = dir.path().join(".provenance/state");
    write_jsonl(
        &state.join("scopes/default/requirements/req.jsonl"),
        &requirement("req_parent"),
    );
    write_jsonl(
        &state.join("scopes/default/threads/threads.jsonl"),
        &thread("thread_valid", "default", "req_parent", 1),
    );

    check(dir.path()).success();
}

fn requirement(id: &str) -> String {
    format!(
        r#"{{"schema_version":1,"scope_id":"default","id":"{id}","statement":"Parent","status":"active"}}"#
    )
}

fn thread(id: &str, scope_id: &str, parent_id: &str, created_at: i64) -> String {
    format!(
        r#"{{"schema_version":1,"scope_id":"{scope_id}","id":"{id}","parent":{{"node_type":"requirement","node_id":"{parent_id}"}},"status":"resolved","created_at":{created_at}}}"#
    )
}

fn init(repo: &Path) {
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
}

fn check(repo: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "check",
            "--repo",
            repo.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
}

fn write_jsonl(path: &Path, records: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, format!("{records}\n")).unwrap();
}
