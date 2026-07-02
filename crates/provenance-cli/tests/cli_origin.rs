use assert_cmd::Command;
use serde_json::Value;

fn provenance_json(args: &[&str]) -> Value {
    let output = Command::cargo_bin("provenance")
        .unwrap()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

fn record_with_id<'a>(records: &'a Value, id: &str) -> &'a Value {
    records
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"] == id)
        .unwrap()
}

#[test]
#[allow(clippy::too_many_lines)]
fn cli_create_commands_preserve_origin_thread_and_message() {
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
    provenance_json(&[
        "requirements",
        "create",
        "--repo",
        &repo,
        "--scope",
        "default",
        "--id",
        "req_origin_seed",
        "--statement",
        "Seed requirement for origin thread",
        "--format",
        "json",
    ]);
    let posted = provenance_json(&[
        "thread",
        "post",
        "--repo",
        &repo,
        "--scope",
        "default",
        "--parent-type",
        "requirement",
        "--parent-id",
        "req_origin_seed",
        "--role",
        "user",
        "--format",
        "json",
        "Promote this conversation into artifacts",
    ]);
    let origin_thread = posted["thread"]["id"].as_str().unwrap().to_string();
    let origin_message = posted["message"]["id"].as_str().unwrap().to_string();

    let source = provenance_json(&[
        "sources",
        "create",
        "--repo",
        &repo,
        "--scope",
        "default",
        "--id",
        "source_origin",
        "--name",
        "Origin Source",
        "--origin-thread",
        &origin_thread,
        "--origin-message",
        &origin_message,
        "--format",
        "json",
    ]);
    let requirement = provenance_json(&[
        "requirements",
        "create",
        "--repo",
        &repo,
        "--scope",
        "default",
        "--id",
        "req_origin_child",
        "--statement",
        "Origin child requirement",
        "--origin-thread",
        &origin_thread,
        "--origin-message",
        &origin_message,
        "--format",
        "json",
    ]);
    let resolution = provenance_json(&[
        "resolutions",
        "create",
        "--repo",
        &repo,
        "--scope",
        "default",
        "--id",
        "res_origin",
        "--title",
        "Origin resolution",
        "--requirement-id",
        "req_origin_child",
        "--position",
        "Use the promoted conversation",
        "--rationale",
        "The thread captured the decision",
        "--origin-thread",
        &origin_thread,
        "--origin-message",
        &origin_message,
        "--format",
        "json",
    ]);
    let rule = provenance_json(&[
        "rules",
        "create",
        "--repo",
        &repo,
        "--scope",
        "default",
        "--id",
        "rule_origin",
        "--rule-code",
        "ORIGIN-001",
        "--requirement-id",
        "req_origin_child",
        "--resolution-id",
        "res_origin",
        "--statement",
        "Artifacts shall keep their origin conversation",
        "--origin-thread",
        &origin_thread,
        "--origin-message",
        &origin_message,
        "--format",
        "json",
    ]);

    for artifact in [&source, &requirement, &resolution, &rule] {
        assert_eq!(artifact["origin_thread"], origin_thread);
        assert_eq!(artifact["origin_message"], origin_message);
    }

    let exported = provenance_json(&[
        "export", "--repo", &repo, "--scope", "default", "--format", "json",
    ]);
    assert_eq!(
        record_with_id(&exported["sources"], "source_origin")["origin_thread"],
        origin_thread
    );
    assert_eq!(
        record_with_id(&exported["requirements"], "req_origin_child")["origin_message"],
        origin_message
    );
    assert_eq!(
        record_with_id(&exported["resolutions"], "res_origin")["origin_thread"],
        origin_thread
    );
    assert_eq!(
        record_with_id(&exported["rules"], "rule_origin")["origin_message"],
        origin_message
    );
}
