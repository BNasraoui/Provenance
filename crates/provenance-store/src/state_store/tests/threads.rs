use super::*;

#[test]
fn list_messages_reads_all_month_shards() {
    let (_dir, store, scope) = seeded_source_requirement_store();
    let first_message = Message {
        schema_version: provenance_core::SchemaVersion(1),
        scope_id: scope.clone(),
        id: StableId::new("msg_july").unwrap(),
        thread_id: StableId::new("thread_source_source_schads").unwrap(),
        role: MessageRole::User,
        body: "July message".into(),
        created_at: 1,
        ai_metadata: None,
    };
    let second_message = Message {
        schema_version: provenance_core::SchemaVersion(1),
        scope_id: scope.clone(),
        id: StableId::new("msg_august").unwrap(),
        thread_id: StableId::new("thread_source_source_schads").unwrap(),
        role: MessageRole::Assistant,
        body: "August message".into(),
        created_at: 2,
        ai_metadata: None,
    };
    let threads_dir = store
        .layout
        .scopes_dir()
        .join(scope.as_str())
        .join("threads");
    std::fs::create_dir_all(&threads_dir).unwrap();
    std::fs::write(
        threads_dir.join("2026-07.jsonl"),
        format!("{}\n", serde_json::to_string(&first_message).unwrap()),
    )
    .unwrap();
    std::fs::write(
        threads_dir.join("2026-08.jsonl"),
        format!("{}\n", serde_json::to_string(&second_message).unwrap()),
    )
    .unwrap();

    let messages = store.list_messages(&scope).unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, first_message.id);
    assert_eq!(messages[1].id, second_message.id);
}
