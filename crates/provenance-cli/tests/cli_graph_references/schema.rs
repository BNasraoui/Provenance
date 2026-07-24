use super::*;

#[test]
fn graph_reference_schemas_are_exposed() {
    let temp = committed_store();
    for artifact in ["graph-reference", "graph-reference-export"] {
        provenance(temp.path())
            .args(["schema", "show", artifact, "--format", "json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"const\": 1"))
            .stdout(predicate::str::contains("additionalProperties"));
    }
}

#[test]
fn emitted_graph_reference_schema_validates_an_issued_reference() {
    let temp = committed_store();
    let reference = issue(temp.path(), &[]);
    let output = provenance(temp.path())
        .args(["schema", "show", "graph-reference", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let shown: Value = serde_json::from_slice(&output).unwrap();
    let schema = shown.get("schema").unwrap();
    let validator = jsonschema::JSONSchema::compile(schema).unwrap();

    assert!(validator.is_valid(&reference));
}

#[test]
fn graph_reference_schema_and_runtime_reject_the_same_boundary_values() {
    let temp = committed_store();
    let output = provenance(temp.path())
        .args(["schema", "show", "graph-reference", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let shown: Value = serde_json::from_slice(&output).unwrap();
    let validator = jsonschema::JSONSchema::compile(shown.get("schema").unwrap()).unwrap();
    let reference = issue(temp.path(), &[]);

    for (name, malformed) in [
        ("41-character-commit", {
            let mut malformed = reference.clone();
            malformed["commit"] = Value::String("0".repeat(41));
            malformed
        }),
        ("uppercase-commit", {
            let mut malformed = reference.clone();
            malformed["commit"] = Value::String("A".repeat(40));
            malformed
        }),
        ("null-correlation", {
            let mut malformed = reference.clone();
            malformed["correlation"] = Value::Null;
            malformed
        }),
        ("whitespace-correlation", {
            let mut malformed = reference;
            malformed["correlation"] = serde_json::json!({"system": "   ", "key": "key"});
            malformed
        }),
    ] {
        assert!(!validator.is_valid(&malformed), "schema accepted {name}");
        let path = temp.path().join(format!("{name}.json"));
        std::fs::write(&path, serde_json::to_vec(&malformed).unwrap()).unwrap();
        provenance(temp.path())
            .args([
                "validate",
                "graph-reference",
                "--input",
                path.to_str().unwrap(),
                "--format",
                "json",
            ])
            .assert()
            .failure();
    }
}
