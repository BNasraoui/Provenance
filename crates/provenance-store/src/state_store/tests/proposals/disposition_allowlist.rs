use super::super::initialized_store;

#[test]
fn scope_validation_does_not_bypass_the_manifest_when_no_allowlist_check_fires() {
    let (_dir, store, scope) = initialized_store();
    std::fs::remove_file(store.layout.manifest_path()).unwrap();

    assert!(store.validate_ideation_scope(&scope).is_err());
}
