use super::{serde_name, PostMessageInput, PostMessageResult, StateStore};
use crate::shards;
use provenance_core::{Message, SchemaVersion, StableId, Thread, ThreadStatus};

impl StateStore {
    pub fn post_thread_message(
        &self,
        input: PostMessageInput,
    ) -> anyhow::Result<PostMessageResult> {
        self.with_repository_publication(|| self.write_thread_message(input))
    }

    fn write_thread_message(&self, input: PostMessageInput) -> anyhow::Result<PostMessageResult> {
        let PostMessageInput {
            scope_id,
            parent,
            role,
            body,
        } = input;
        anyhow::ensure!(!body.trim().is_empty(), "message body must not be empty");
        let threads_path = shards::threads_path(&self.layout, &scope_id);
        let thread = self.mutate_jsonl_records(&threads_path, |threads: &mut Vec<Thread>| {
            let matching: Vec<_> = threads
                .iter()
                .filter(|thread| thread.parent == parent)
                .cloned()
                .collect();
            let thread = if let Some(canonical) =
                provenance_core::threads::choose_canonical_active_thread(&matching)
            {
                let canonical = canonical.clone();
                for thread in threads.iter_mut() {
                    if thread.parent == parent
                        && thread.status == ThreadStatus::Active
                        && thread.id != canonical.id
                    {
                        thread.status = ThreadStatus::Archived;
                    }
                }
                canonical
            } else {
                let thread = Thread {
                    schema_version: SchemaVersion(1),
                    scope_id: scope_id.clone(),
                    id: StableId::new(format!(
                        "thread_{}_{}",
                        serde_name(&parent.node_type)?,
                        parent.node_id.as_str()
                    ))?,
                    parent: parent.clone(),
                    status: ThreadStatus::Active,
                    created_at: 1,
                };
                threads.push(thread.clone());
                thread
            };
            threads.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(thread)
        })?;

        let messages_path = shards::messages_path(&self.layout, &scope_id);
        let message =
            self.mutate_jsonl_records(&messages_path, |messages: &mut Vec<Message>| {
                let created_at = messages
                    .iter()
                    .map(|message| message.created_at)
                    .max()
                    .unwrap_or(0)
                    + 1;
                let message = Message {
                    schema_version: SchemaVersion(1),
                    scope_id: scope_id.clone(),
                    id: StableId::new(format!("msg_{created_at:06}"))?,
                    thread_id: thread.id.clone(),
                    role,
                    body,
                    created_at,
                    ai_metadata: None,
                };
                messages.push(message.clone());
                messages.sort_by(|a, b| {
                    a.created_at
                        .cmp(&b.created_at)
                        .then(a.id.as_str().cmp(b.id.as_str()))
                });
                Ok(message)
            })?;
        Ok(PostMessageResult { thread, message })
    }
}
