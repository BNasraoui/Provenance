use assert_cmd::Command;

#[test]
fn altered_replaced_or_omitted_shipped_disposition_audit_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let baseline = export_shipped(&dir);
    for attack in ["altered", "replaced", "omitted"] {
        let mut value = baseline.clone();
        let dispositions = value["dispositions"].as_array_mut().unwrap();
        match attack {
            "altered" => {
                dispositions[0]["rationale"] = serde_json::json!(format!(
                    "{}x",
                    dispositions[0]["rationale"].as_str().unwrap()
                ));
            }
            "replaced" => {
                dispositions[0]["id"] = serde_json::json!("disposition_replacement");
            }
            "omitted" => {
                dispositions.remove(0);
            }
            _ => unreachable!(),
        }
        let input = dir.path().join(format!("{attack}.json"));
        std::fs::write(&input, serde_json::to_vec(&value).unwrap()).unwrap();
        let repo = dir.path().join(format!("repo-{attack}"));
        init(&repo);
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                "import",
                "--repo",
                repo.to_str().unwrap(),
                "--scope",
                "default",
                "--input",
                input.to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                "frozen shipped-v1 disposition audit",
            ));
    }
}

#[test]
fn exact_shipped_promotion_decisions_export_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let mut legacy = export_shipped(&dir);
    legacy.as_object_mut().unwrap().remove("assertion_records");
    let dispositions = legacy
        .as_object_mut()
        .unwrap()
        .remove("dispositions")
        .unwrap();
    legacy["promotion_decisions"] = dispositions;
    let input = dir.path().join("legacy.json");
    std::fs::write(&input, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let repo = dir.path().join("repo");
    init(&repo);
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            input.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn import_cannot_omit_entire_existing_shipped_legacy_terminal_set() {
    let dir = tempfile::tempdir().unwrap();
    let mut shipped = export_shipped(&dir);
    let repo = dir.path().join("repo");
    init(&repo);
    let complete = dir.path().join("complete.json");
    std::fs::write(&complete, serde_json::to_vec(&shipped).unwrap()).unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            complete.to_str().unwrap(),
        ])
        .assert()
        .success();

    shipped["proposal_cards"]
        .as_array_mut()
        .unwrap()
        .retain(|proposal| proposal["promotion_state"].as_str() == Some("proposed"));
    shipped["dispositions"] = serde_json::json!([]);
    let omitted = dir.path().join("omitted-all.json");
    std::fs::write(&omitted, serde_json::to_vec(&shipped).unwrap()).unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--input",
            omitted.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("immutable proposal"));
}

#[test]
fn ambiguous_or_unknown_export_fields_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let baseline = export_shipped(&dir);
    for attack in ["both", "unknown"] {
        let mut value = baseline.clone();
        if attack == "both" {
            value["promotion_decisions"] = value["dispositions"].clone();
        } else {
            value["unexpected"] = serde_json::json!(true);
        }
        let input = dir.path().join(format!("{attack}.json"));
        std::fs::write(&input, serde_json::to_vec(&value).unwrap()).unwrap();
        let repo = dir.path().join(format!("repo-{attack}"));
        init(&repo);
        Command::cargo_bin("provenance")
            .unwrap()
            .args([
                "import",
                "--repo",
                repo.to_str().unwrap(),
                "--scope",
                "default",
                "--input",
                input.to_str().unwrap(),
            ])
            .assert()
            .failure();
    }
}

fn export_shipped(dir: &tempfile::TempDir) -> serde_json::Value {
    let output = dir.path().join("shipped.json");
    let shipped = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "export",
            "--repo",
            shipped.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap()
}

fn init(repo: &std::path::Path) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--disposition-actor-id",
            "codex-review-panel-gpt55-medium",
        ])
        .assert()
        .success();
}
