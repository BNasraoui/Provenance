use crate::handlers::check::index::CheckIndex;
use crate::handlers::check::references::{check_scoped_reference, node_type_name};
use provenance_core::{Message, ScopeId, Thread};
use provenance_store::state_store::StateStore;

pub(super) struct Records {
    threads: Vec<Thread>,
    messages: Vec<Message>,
}

impl Records {
    pub(super) fn load(store: &StateStore, scope_id: &ScopeId) -> anyhow::Result<Self> {
        Ok(Self {
            threads: store.list_threads(scope_id)?,
            messages: store.list_messages(scope_id)?,
        })
    }

    pub(super) fn validate_scope_ownership(
        &self,
        loaded_scope_id: &ScopeId,
        findings: &mut Vec<String>,
    ) {
        macro_rules! check_records {
            ($records:expr, $record_type:literal) => {
                for record in $records {
                    super::check_scope_ownership(
                        loaded_scope_id,
                        &record.scope_id,
                        $record_type,
                        &record.id,
                        findings,
                    );
                }
            };
        }

        check_records!(&self.threads, "thread");
        check_records!(&self.messages, "message");
    }

    pub(super) fn add_to(&self, index: &mut CheckIndex) {
        for thread in &self.threads {
            index.add_node(&thread.scope_id, "thread", &thread.id);
        }
        for message in &self.messages {
            index.add_node(&message.scope_id, "message", &message.id);
        }
    }

    pub(super) fn validate(
        &self,
        index: &CheckIndex,
        scope_id: &ScopeId,
        dangling: &mut Vec<String>,
    ) {
        for thread in &self.threads {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &format!("thread {}", thread.id.as_str()),
                "parent",
                node_type_name(thread.parent.node_type),
                &thread.parent.node_id,
            );
        }
        for message in &self.messages {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &format!("message {}", message.id.as_str()),
                "thread",
                "thread",
                &message.thread_id,
            );
        }
    }
}
