use predicates::str::contains;

use crate::provenance::provenance;

pub fn init(repo: &str) {
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

pub fn create_source_and_requirement(repo: &str) {
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

pub fn create_topic(repo: &str) {
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
