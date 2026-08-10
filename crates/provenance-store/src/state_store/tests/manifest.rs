use super::initialized_store;
use provenance_core::SchemaVersion;
use provenance_macros::verifies;

#[test]
#[verifies("rule_schema_version_one", examples)]
fn manifest_rejects_unsupported_schema_version() {
    let (_dir, store, _scope) = initialized_store();
    let mut manifest = store.manifest().unwrap();
    manifest.schema_version = SchemaVersion(2);
    std::fs::write(
        store.layout.manifest_path(),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let error = store.manifest().unwrap_err();
    assert_eq!(error.to_string(), "manifest schema_version must be 1");
}
