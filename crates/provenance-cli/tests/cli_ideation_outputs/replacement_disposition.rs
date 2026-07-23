use super::support::{create_source, init_repo};
use assert_cmd::Command;

#[test]
#[allow(clippy::too_many_lines)]
fn ideation_create_replace_updates_existing_records() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    init_repo(&repo);
    create_source(&repo, "source_codebase");

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "contributions",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Original finding.",
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Initial run.",
            "--format",
            "json",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "contributions",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Duplicate finding.",
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Duplicate run.",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "contributions",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "contrib_extract_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--participant-slot",
            "extract_auth",
            "--stance",
            "support",
            "--strongest-finding",
            "Updated finding.",
            "--uncertainty-level",
            "medium",
            "--uncertainty-rationale",
            "Second run.",
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated finding."));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "synthesis-packets",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "synth_backtrace_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--summary",
            "Original synthesis.",
            "--format",
            "json",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "synthesis-packets",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "synth_backtrace_auth",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--summary",
            "Updated synthesis.",
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated synthesis."));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "prop_req_publish_requires_worker",
            "--proposal-key",
            "backtrace/auth/publish_requires_worker",
            "--proposal-type",
            "requirement_candidate",
            "--title",
            "Original proposal",
            "--summary",
            "Original summary.",
            "--confidence",
            "0.5",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--source-id",
            "source_codebase",
            "--promotion-state",
            "proposed",
            "--format",
            "json",
        ])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "create",
            "--repo",
            &repo,
            "--scope",
            "default",
            "--id",
            "prop_req_publish_requires_worker",
            "--proposal-key",
            "backtrace/auth/publish_requires_worker",
            "--proposal-type",
            "requirement_candidate",
            "--title",
            "Updated proposal",
            "--summary",
            "Updated summary.",
            "--confidence",
            "0.9",
            "--target-type",
            "source",
            "--target-id",
            "source_codebase",
            "--source-id",
            "source_codebase",
            "--promotion-state",
            "proposed",
            "--replace",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("immutable"));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export", "--repo", &repo, "--scope", "default", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated finding."))
        .stdout(predicates::str::contains("Updated synthesis."))
        .stdout(predicates::str::contains("Original proposal"))
        .stdout(predicates::str::contains(r#""confidence": 0.5"#));
}
