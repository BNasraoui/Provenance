use super::{serde_name, PostMessageInput, PostMessageResult, StateStore};
use crate::{jsonl, shards};
use provenance_core::{Message, SchemaVersion, StableId, Thread, ThreadStatus};

impl StateStore {
    pub fn post_thread_message(
        &self,
        input: PostMessageInput,
    ) -> anyhow::Result<PostMessageResult> {
        anyhow::ensure!(
            !input.body.trim().is_empty(),
            "message body must not be empty"
        );
        let mut threads = self.list_threads(&input.scope_id)?;
        let matching: Vec<_> = threads
            .iter()
            .filter(|thread| thread.parent == input.parent)
            .cloned()
            .collect();
        let thread = if let Some(canonical) =
            provenance_core::threads::choose_canonical_active_thread(&matching)
        {
            let canonical = canonical.clone();
            for thread in &mut threads {
                if thread.parent == input.parent
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
                scope_id: input.scope_id.clone(),
                id: StableId::new(format!(
                    "thread_{}_{}",
                    serde_name(&input.parent.node_type)?,
                    input.parent.node_id.as_str()
                ))?,
                parent: input.parent.clone(),
                status: ThreadStatus::Active,
                created_at: 1,
            };
            threads.push(thread.clone());
            thread
        };
        threads.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(
            &shards::threads_path(&self.layout, &input.scope_id),
            &threads,
        )?;

        let mut messages = self.list_messages(&input.scope_id)?;
        let created_at = messages
            .iter()
            .map(|message| message.created_at)
            .max()
            .unwrap_or(0)
            + 1;
        let message = Message {
            schema_version: SchemaVersion(1),
            scope_id: input.scope_id.clone(),
            id: StableId::new(format!("msg_{created_at:06}"))?,
            thread_id: thread.id.clone(),
            role: input.role,
            body: input.body,
            created_at,
            ai_metadata: None,
        };
        messages.push(message.clone());
        messages.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then(a.id.as_str().cmp(b.id.as_str()))
        });
        jsonl::write_jsonl_atomic(
            &shards::messages_path(&self.layout, &input.scope_id),
            &messages,
        )?;
        Ok(PostMessageResult { thread, message })
    }
}
