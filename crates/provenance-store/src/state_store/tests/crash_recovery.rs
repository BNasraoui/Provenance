use super::seeded_requirement_store;
use crate::state_store::StateStore;
use crate::transaction::StateTransaction;
use provenance_core::{
    Edge, EdgeType, Message, MessageRole, NodeType, Requirement, SchemaVersion, ScopeId, StableId,
    Topic, TopicStatus,
};
use serde::Serialize;

const OLD: &str = "old_generation";
const INTERRUPTED: &str = "interrupted_generation";

struct InterruptedGeneration {
    _dir: tempfile::TempDir,
    store: StateStore,
    scope: ScopeId,
}

impl InterruptedGeneration {
    fn new() -> Self {
        let (dir, store, scope) = seeded_requirement_store();
        let edge_paths = [
            store.layout.edges_dir().join("edges-00.jsonl"),
            store.layout.edges_dir().join("edges-01.jsonl"),
        ];
        let message_paths = [
            store
                .layout
                .scopes_dir()
                .join(scope.as_str())
                .join("threads/2026-07.jsonl"),
            store
                .layout
                .scopes_dir()
                .join(scope.as_str())
                .join("threads/2026-08.jsonl"),
        ];
        let topic_path = crate::shards::topics_path(&store.layout, &scope);
        for (index, path) in edge_paths.iter().enumerate() {
            write_records(path, &[edge(&scope, OLD, index)]);
        }
        for (index, path) in message_paths.iter().enumerate() {
            write_records(path, &[message(&scope, OLD, index)]);
        }
        write_records(&topic_path, &[topic(&scope, OLD)]);

        let requirement_path = crate::shards::requirements_path(&store.layout, &scope);
        let mut transaction =
            StateTransaction::new(store.layout.state_transaction_journal_path(), None);
        transaction
            .mutate_jsonl(&requirement_path, |requirements: &mut Vec<Requirement>| {
                requirements[0].statement = INTERRUPTED.into();
                Ok(())
            })
            .unwrap();
        for (index, path) in edge_paths.iter().enumerate() {
            transaction
                .replace_jsonl(path, &[edge(&scope, INTERRUPTED, index)])
                .unwrap();
        }
        for (index, path) in message_paths.iter().enumerate() {
            transaction
                .replace_jsonl(path, &[message(&scope, INTERRUPTED, index)])
                .unwrap();
        }
        transaction
            .replace_jsonl(&topic_path, &[topic(&scope, INTERRUPTED)])
            .unwrap();
        transaction.simulate_crash_after_activations(5).unwrap();

        assert_eq!(
            read_records::<Requirement>(&requirement_path)[0].statement,
            INTERRUPTED
        );
        assert_eq!(
            read_records::<Edge>(&edge_paths[1])[0].label.as_deref(),
            Some(INTERRUPTED)
        );
        assert_eq!(
            read_records::<Message>(&message_paths[1])[0].body,
            INTERRUPTED
        );
        assert_eq!(read_records::<Topic>(&topic_path)[0].title, OLD);
        assert!(store.layout.state_transaction_journal_path().exists());
        Self {
            _dir: dir,
            store,
            scope,
        }
    }

    fn assert_recovered(&self) {
        assert!(!self.layout().state_transaction_journal_path().exists());
        assert_eq!(
            self.store.list_requirements(&self.scope).unwrap()[0].statement,
            "Overtime"
        );
        assert!(self
            .store
            .list_edges()
            .unwrap()
            .iter()
            .all(|edge| edge.label.as_deref() == Some(OLD)));
        assert!(self
            .store
            .list_messages(&self.scope)
            .unwrap()
            .iter()
            .all(|message| message.body == OLD));
        assert_eq!(self.store.list_topics(&self.scope).unwrap()[0].title, OLD);
    }

    fn layout(&self) -> &crate::layout::ProvenanceLayout {
        &self.store.layout
    }
}

#[test]
fn ordinary_public_list_reader_recovers_interrupted_generation() {
    let fixture = InterruptedGeneration::new();

    let requirements = fixture.store.list_requirements(&fixture.scope).unwrap();

    assert_eq!(requirements[0].statement, "Overtime");
    fixture.assert_recovered();
}

#[test]
fn physical_edge_shard_reader_recovers_interrupted_generation() {
    let fixture = InterruptedGeneration::new();

    let edges = fixture.store.list_edges().unwrap();

    assert_eq!(edges.len(), 2);
    assert!(edges.iter().all(|edge| edge.label.as_deref() == Some(OLD)));
    fixture.assert_recovered();
}

#[test]
fn physical_message_shard_reader_recovers_interrupted_generation() {
    let fixture = InterruptedGeneration::new();

    let messages = fixture.store.list_messages(&fixture.scope).unwrap();

    assert_eq!(messages.len(), 2);
    assert!(messages.iter().all(|message| message.body == OLD));
    fixture.assert_recovered();
}

#[test]
fn repository_snapshot_recovers_interrupted_generation() {
    let fixture = InterruptedGeneration::new();

    let snapshot = fixture.store.repository_snapshot().unwrap();

    assert_eq!(snapshot.scopes[0].requirements[0].statement, "Overtime");
    assert!(snapshot
        .edges
        .iter()
        .all(|edge| edge.label.as_deref() == Some(OLD)));
    assert!(snapshot.scopes[0]
        .messages
        .iter()
        .all(|message| message.body == OLD));
    assert_eq!(snapshot.scopes[0].topics[0].title, OLD);
    fixture.assert_recovered();
}

fn edge(scope: &ScopeId, generation: &str, index: usize) -> Edge {
    Edge {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: StableId::new(format!("edge_{generation}_{index}")).unwrap(),
        edge_type: EdgeType::DependsOn,
        from_type: NodeType::Requirement,
        from_id: StableId::new("req_overtime").unwrap(),
        to_type: NodeType::Requirement,
        to_id: StableId::new("req_overtime").unwrap(),
        label: Some(generation.into()),
    }
}

fn message(scope: &ScopeId, generation: &str, index: usize) -> Message {
    Message {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: StableId::new(format!("msg_{generation}_{index}")).unwrap(),
        thread_id: StableId::new("thread_recovery").unwrap(),
        role: MessageRole::User,
        body: generation.into(),
        created_at: i64::try_from(index).unwrap(),
        ai_metadata: None,
    }
}

fn topic(scope: &ScopeId, generation: &str) -> Topic {
    Topic {
        schema_version: SchemaVersion(1),
        scope_id: scope.clone(),
        id: StableId::new(format!("topic_{generation}")).unwrap(),
        requirement_id: StableId::new("req_overtime").unwrap(),
        title: generation.into(),
        status: TopicStatus::Open,
        claimed_by: None,
        claimed_at: None,
        links: Vec::new(),
    }
}

fn write_records<T: Serialize>(path: &camino::Utf8Path, records: &[T]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let contents = records
        .iter()
        .map(|record| serde_json::to_string(record).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, format!("{contents}\n")).unwrap();
}

fn read_records<T: serde::de::DeserializeOwned>(path: &camino::Utf8Path) -> Vec<T> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
