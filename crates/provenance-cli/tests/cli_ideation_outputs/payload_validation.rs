use super::support::{create_source, init_repo};
use assert_cmd::Command;

#[test]
fn ideation_json_flags_accept_at_file_payloads() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_string_lossy().to_string();
    init_repo(&repo);
    create_source(&repo, "source_codebase");

    let evidence_path = dir.path().join("evidence.json");
    let claims_path = dir.path().join("claims.json");
    std::fs::write(
        &evidence_path,
        r#"[{"reference_id":"evidence_auth_guard","evidence_type":"artifact","summary":"Guard rejects missing worker","file_path":"src/auth.rs","line":12}]"#,
    )
    .unwrap();
    std::fs::write(
        &claims_path,
        r#"[{"claim_id":"claim_auth_guard","statement":"Publishing requires an assigned worker.","evidence_type":"artifact","evidence_reference_ids":["evidence_auth_guard"],"confidence":0.91}]"#,
    )
    .unwrap();
    let evidence_arg = format!("@{}", evidence_path.to_string_lossy());
    let claims_arg = format!("@{}", claims_path.to_string_lossy());

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
            "Publishing is guarded by worker assignment.",
            "--evidence-json",
            &evidence_arg,
            "--claims-json",
            &claims_arg,
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Direct guard evidence.",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("claim_auth_guard"))
        .stdout(predicates::str::contains("evidence_auth_guard"));
}

#[test]
fn ideation_nested_ids_are_semantically_validated() {
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
            "Publishing is guarded by worker assignment.",
            "--evidence-json",
            r#"[{"reference_id":"evidence/auth","evidence_type":"artifact","summary":"Bad id","file_path":"src/auth.rs","line":12}]"#,
            "--claims-json",
            r#"[{"claim_id":"claim_auth_guard","statement":"Publishing requires an assigned worker.","evidence_type":"artifact","evidence_reference_ids":["evidence/auth"]}]"#,
            "--uncertainty-level",
            "low",
            "--uncertainty-rationale",
            "Direct guard evidence.",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("stable id"));
}
