use crate::{Thread, ThreadStatus};

pub fn choose_canonical_active_thread(threads: &[Thread]) -> Option<&Thread> {
    threads
        .iter()
        .filter(|thread| thread.status == ThreadStatus::Active)
        .min_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then(a.id.as_str().cmp(b.id.as_str()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeType, SchemaVersion, ScopeId, StableId, ThreadParent};

    fn thread(id: &str, status: ThreadStatus, created_at: i64) -> Thread {
        Thread {
            schema_version: SchemaVersion(1),
            scope_id: ScopeId::new("default").unwrap(),
            id: StableId::new(id).unwrap(),
            parent: ThreadParent {
                node_type: NodeType::Rule,
                node_id: StableId::new("rule_schads_pay_001").unwrap(),
            },
            status,
            created_at,
        }
    }

    #[test]
    fn threads_choose_oldest_active_thread_as_canonical() {
        let threads = vec![
            thread("thread_new", ThreadStatus::Active, 20),
            thread("thread_old", ThreadStatus::Active, 10),
            thread("thread_archived", ThreadStatus::Archived, 1),
        ];

        assert_eq!(
            choose_canonical_active_thread(&threads)
                .unwrap()
                .id
                .as_str(),
            "thread_old"
        );
    }
}
