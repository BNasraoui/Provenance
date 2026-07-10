use super::ScopeRecords;
use crate::handlers::check::index::CheckIndex;
use crate::handlers::check::references::{check_scoped_reference, node_type_name};
use provenance_core::ScopeId;

pub(in crate::handlers::check) fn validate(
    records: &ScopeRecords,
    index: &CheckIndex,
    scope_id: &ScopeId,
    dangling: &mut Vec<String>,
) {
    for binding in &records.service_bindings {
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
    for thread in &records.threads {
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
    for message in &records.messages {
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
