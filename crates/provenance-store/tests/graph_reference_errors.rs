use provenance_core::{Manifest, RepoPathPrefix, ScopeId};
use provenance_store::{
    graph_reference::{
        ExactExport, ExternalCorrelation, GraphReference, GraphReferenceError, GraphReferences,
    },
    layout::ProvenanceLayout,
};
use serde_json::json;

fn git(repo: &std::path::Path, args: &[&str]) {
    assert!(
        std::process::Command::new("git")
            .args(args)
            .current_dir(repo)
            .status()
            .unwrap()
            .success(),
        "git {args:?} failed"
    );
}

fn committed_store() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_vec(&Manifest::default_with_scope(
            ScopeId::new("default").unwrap(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["add", ".provenance/state"]);
    git(
        temp.path(),
        &[
            "-c",
            "user.name=Graph Reference Test",
            "-c",
            "user.email=graph-reference@example.invalid",
            "commit",
            "-qm",
            "canonical graph",
        ],
    );
    temp
}

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
fn store_api_rejects_invalid_reference_contract() {
    let temp = committed_store();
    let root = camino::Utf8Path::from_path(temp.path()).unwrap();
    let references = GraphReferences::open(root).unwrap();
    let reference = references.issue("default", None, None).unwrap();

    let mut unsupported = reference.clone();
    unsupported.schema_version = 2;
    assert!(matches!(
        references.show(&unsupported),
        Err(GraphReferenceError::Incomplete { .. })
    ));
    assert!(matches!(
        references.verify(&unsupported),
        Err(GraphReferenceError::Incomplete { .. })
    ));
    assert!(matches!(
        references.exact_export(&unsupported),
        Err(GraphReferenceError::Incomplete { .. })
    ));

    let mut invalid_correlation = reference;
    invalid_correlation.correlation = Some(ExternalCorrelation {
        system: " ".into(),
        key: "issue-35".into(),
    });
    assert!(matches!(
        references.show(&invalid_correlation),
        Err(GraphReferenceError::Incomplete { .. })
    ));
    assert!(matches!(
        references.verify(&invalid_correlation),
        Err(GraphReferenceError::Incomplete { .. })
    ));
    assert!(matches!(
        references.exact_export(&invalid_correlation),
        Err(GraphReferenceError::Incomplete { .. })
    ));
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

#[test]
fn exact_export_rejects_collaboration_metadata() {
    let document = json!({
        "schema_version": 1,
        "operation": "exact-export",
        "reference_id": format!("grf1_{}", "0".repeat(64)),
        "graph": {
            "schema_version": 1,
            "scope": {"id": "default", "path_prefix": "."},
            "sources": [{
                "schema_version": 1,
                "scope_id": "default",
                "id": "source_policy",
                "name": "Policy",
                "source_type": "policy",
                "url": null,
                "origin_thread": "thread_private"
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
    assert!(error.to_string().contains("origin_thread"));
}
