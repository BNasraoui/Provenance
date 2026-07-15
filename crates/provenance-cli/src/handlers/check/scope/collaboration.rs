use crate::handlers::check::index::CheckIndex;
use crate::handlers::check::references::{check_scoped_reference, node_type_name};
use provenance_core::{Message, ScopeId, Service, ServiceBinding, Thread};
use provenance_store::state_store::ScopeSnapshot;

pub(super) struct Records {
    services: Vec<Service>,
    service_bindings: Vec<ServiceBinding>,
    threads: Vec<Thread>,
    messages: Vec<Message>,
}

impl Records {
    pub(super) fn load(snapshot: &mut ScopeSnapshot) -> Self {
        Self {
            services: std::mem::take(&mut snapshot.services),
            service_bindings: std::mem::take(&mut snapshot.service_bindings),
            threads: std::mem::take(&mut snapshot.threads),
            messages: std::mem::take(&mut snapshot.messages),
        }
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

        check_records!(&self.services, "service");
        check_records!(&self.service_bindings, "service binding");
        check_records!(&self.threads, "thread");
        check_records!(&self.messages, "message");
    }

    pub(super) fn add_to(&self, index: &mut CheckIndex) {
        for service in &self.services {
            index.add_node(&service.scope_id, "service", &service.id);
        }
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
        for binding in &self.service_bindings {
            let owner = format!("service binding {}", binding.id.as_str());
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &owner,
                "rule",
                "rule",
                &binding.rule_id,
            );
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &owner,
                "service",
                "service",
                &binding.service_id,
            );
        }
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
