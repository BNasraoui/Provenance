use camino::Utf8PathBuf;
use provenance_core::{
    Manifest, Message, MessageRole, RepoPathPrefix, SchemaVersion, ScopeId, StableId,
};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{ScopeReplacement, StateStore},
};

fn initialized_store(root: &camino::Utf8Path) -> StateStore {
    let layout = ProvenanceLayout::new(root.to_path_buf());
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            ScopeId::new("default").unwrap(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    StateStore::new(layout)
}

#[test]
fn public_scope_replacement_derives_paths_from_the_store_layout() {
    let dir = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let unrelated = root.join("caller-selected.jsonl");
    std::fs::write(&unrelated, "do not touch\n").unwrap();
    let store = initialized_store(&root);

    store
        .replace_scope(
            &ScopeId::new("default").unwrap(),
            &ScopeReplacement::default(),
        )
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(unrelated).unwrap(),
        "do not touch\n"
    );
    assert!(root
        .join(".provenance/state/scopes/default/sources/source.jsonl")
        .exists());
}

#[test]
fn public_scope_replacement_rejects_scope_absent_from_manifest_before_writes() {
    let dir = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let store = initialized_store(&root);
    let absent = ScopeId::new("absent").unwrap();

    let error = store
        .replace_scope(&absent, &ScopeReplacement::default())
        .unwrap_err();

    assert!(error.to_string().contains("absent from manifest"));
    assert!(!root.join(".provenance/state/scopes/absent").exists());
    assert!(!root.join(".provenance/state/edges").exists());
}

fn message(scope_id: ScopeId, id: &str) -> Message {
    Message {
        schema_version: SchemaVersion(1),
        scope_id,
        id: StableId::new(id).unwrap(),
        thread_id: StableId::new("thread_import").unwrap(),
        role: MessageRole::User,
        body: id.into(),
        created_at: 1,
        ai_metadata: None,
    }
}

fn write_messages(path: &camino::Utf8Path, messages: &[Message]) {
    let contents = messages
        .iter()
        .map(|message| serde_json::to_string(message).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, format!("{contents}\n")).unwrap();
}

#[test]
fn scope_replacement_cleans_every_message_month_shard_only() {
    let dir = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let store = initialized_store(&root);
    let selected = ScopeId::new("default").unwrap();
    let foreign = ScopeId::new("foreign").unwrap();
    let threads = root.join(".provenance/state/scopes/default/threads");
    std::fs::create_dir_all(&threads).unwrap();
    write_messages(
        &threads.join("2026-06.jsonl"),
        &[
            message(selected.clone(), "stale_june"),
            message(foreign.clone(), "foreign_june"),
        ],
    );
    write_messages(
        &threads.join("2026-07.jsonl"),
        &[
            message(selected.clone(), "stale_canonical"),
            message(foreign, "foreign_canonical"),
        ],
    );
    let non_month = threads.join("messages.jsonl");
    std::fs::write(&non_month, "not message JSON\n").unwrap();

    store
        .replace_scope(
            &selected,
            &ScopeReplacement {
                messages: vec![message(selected.clone(), "imported")],
                ..ScopeReplacement::default()
            },
        )
        .unwrap();

    let june = std::fs::read_to_string(threads.join("2026-06.jsonl")).unwrap();
    let canonical = std::fs::read_to_string(threads.join("2026-07.jsonl")).unwrap();
    assert!(!june.contains("stale_june"));
    assert!(june.contains("foreign_june"));
    assert!(!canonical.contains("stale_canonical"));
    assert!(canonical.contains("foreign_canonical"));
    assert!(canonical.contains("imported"));
    assert!(!june.contains("imported"));
    assert_eq!(
        std::fs::read_to_string(non_month).unwrap(),
        "not message JSON\n"
    );
}
