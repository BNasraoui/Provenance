use provenance_store::graph_reference::{ExactExport, GraphReference, GraphReferenceError};
use serde_json::json;

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

#[test]
fn exact_export_rejects_unsupported_record_schema_versions() {
    let document = json!({
        "schema_version": 1,
        "operation": "exact-export",
        "reference_id": format!("grf1_{}", "0".repeat(64)),
        "graph": {
            "schema_version": 1,
            "scope": {"id": "default", "path_prefix": "."},
            "sources": [{
                "schema_version": 2,
                "scope_id": "default",
                "id": "source_policy",
                "name": "Policy",
                "source_type": "policy",
                "url": null
            }],
            "domains": [],
            "requirements": [],
            "boundaries": [],
            "topics": [],
            "questions": [],
            "resolutions": [],
            "rules": [],
            "services": [],
            "service_bindings": [],
            "edges": []
        }
    });

    let error = ExactExport::from_json(&serde_json::to_vec(&document).unwrap()).unwrap_err();
    assert!(matches!(error, GraphReferenceError::Incomplete { .. }));
    assert!(error.to_string().contains("source 'source_policy'"));
    assert!(error.to_string().contains("schema_version 2"));
}
