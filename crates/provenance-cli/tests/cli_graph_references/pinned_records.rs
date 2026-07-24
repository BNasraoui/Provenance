use super::*;

#[test]
fn issue_rejects_unsupported_pinned_record_schema_versions() {
    let temp = committed_store();
    provenance(temp.path())
        .args([
            "sources",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "source_v2",
            "--name",
            "Future source",
        ])
        .assert()
        .success();
    let source_path = temp
        .path()
        .join(".provenance/state/scopes/default/sources/source.jsonl");
    let source = std::fs::read_to_string(&source_path).unwrap();
    std::fs::write(
        &source_path,
        source.replace("\"schema_version\":1", "\"schema_version\":2"),
    )
    .unwrap();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "add unsupported source"]);

    provenance(temp.path())
        .args([
            "graph-reference",
            "issue",
            "--repo",
            ".",
            "--scope",
            "default",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "source 'source_v2' has unsupported schema_version 2",
        ));
}

#[test]
fn issue_rejects_unknown_fields_in_pinned_graph_records() {
    let temp = committed_store();
    provenance(temp.path())
        .args([
            "sources",
            "create",
            "--repo",
            ".",
            "--scope",
            "default",
            "--id",
            "source_typo",
            "--name",
            "Typo source",
        ])
        .assert()
        .success();
    let source_path = temp
        .path()
        .join(".provenance/state/scopes/default/sources/source.jsonl");
    let source = std::fs::read_to_string(&source_path).unwrap();
    std::fs::write(
        &source_path,
        source.replace(
            "\"name\":\"Typo source\"",
            "\"name\":\"Typo source\",\"naem\":\"lost\"",
        ),
    )
    .unwrap();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "add malformed source"]);

    provenance(temp.path())
        .args([
            "graph-reference",
            "issue",
            "--repo",
            ".",
            "--scope",
            "default",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown field"));
}

#[test]
fn selected_scope_ignores_future_data_from_another_scope() {
    let temp = committed_store();
    let manifest_path = temp.path().join(".provenance/state/manifest.json");
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["scopes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "future",
            "path_prefix": "future",
            "future_field": true
        }));
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    std::fs::write(
        temp.path().join(".provenance/state/edges/edges-99.jsonl"),
        r#"{"schema_version":2,"scope_id":"future","id":"edge_future","future_field":true}
"#,
    )
    .unwrap();
    git(temp.path(), &["add", ".provenance/state"]);
    git(temp.path(), &["commit", "-qm", "add future scope data"]);

    let reference = issue(temp.path(), &[]);
    let reference_path = write_reference(temp.path(), &reference);
    provenance(temp.path())
        .args([
            "graph-reference",
            "exact-export",
            "--repo",
            ".",
            "--reference",
            reference_path.to_str().unwrap(),
        ])
        .assert()
        .success();
}
