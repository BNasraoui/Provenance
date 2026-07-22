use provenance_store::graph_reference::{GraphReference, GraphReferenceError};

#[test]
fn malformed_reference_is_typed_as_incomplete() {
    let error = GraphReference::from_json(br#"{"schema_version":1}"#).unwrap_err();
    assert!(matches!(error, GraphReferenceError::Incomplete { .. }));
}

#[test]
fn unsupported_reference_version_is_typed_as_incomplete() {
    let document = br#"{
        "schema_version": 2,
        "reference_id": "grf1_x",
        "repository_id": "git1_x",
        "store_path": ".provenance/state",
        "scope_id": "default",
        "commit": "0000000000000000000000000000000000000000",
        "graph_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    }"#;
    let error = GraphReference::from_json(document).unwrap_err();
    assert!(matches!(error, GraphReferenceError::Incomplete { .. }));
}
