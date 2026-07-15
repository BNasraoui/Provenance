use super::StateStore;
use crate::shards;
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_core::{
    Boundary, Contribution, Domain, Edge, Manifest, Message, PromotionDecisionRecord, ProposalCard,
    Question, Requirement, Resolution, Rule, ScopeId, Service, ServiceBinding, Source,
    SynthesisPacket, Thread, Topic,
};
use serde::de::DeserializeOwned;

macro_rules! scoped_reader {
    ($public:ident, $unlocked:ident, $record:ty, $path:ident) => {
        pub fn $public(&self, scope: &ScopeId) -> anyhow::Result<Vec<$record>> {
            self.read_generation(|| self.$unlocked(scope))
        }

        pub(crate) fn $unlocked(&self, scope: &ScopeId) -> anyhow::Result<Vec<$record>> {
            read_jsonl(&shards::$path(&self.layout, scope))
        }
    };
}

impl StateStore {
    pub fn manifest(&self) -> anyhow::Result<Manifest> {
        self.read_generation(|| self.manifest_unlocked())
    }

    pub(crate) fn manifest_unlocked(&self) -> anyhow::Result<Manifest> {
        Ok(serde_json::from_str(&std::fs::read_to_string(
            self.layout.manifest_path(),
        )?)?)
    }

    pub fn list_scope_directories(&self) -> anyhow::Result<Vec<String>> {
        self.read_generation(|| self.list_scope_directories_unlocked())
    }

    pub(crate) fn list_scope_directories_unlocked(&self) -> anyhow::Result<Vec<String>> {
        let scopes_dir = self.layout.scopes_dir();
        if !scopes_dir.exists() {
            return Ok(Vec::new());
        }
        let mut directories = Vec::new();
        for entry in std::fs::read_dir(scopes_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                directories.push(entry.file_name().into_string().map_err(|name| {
                    anyhow::anyhow!("non-UTF-8 scope directory name: {}", name.to_string_lossy())
                })?);
            }
        }
        directories.sort();
        Ok(directories)
    }

    scoped_reader!(list_sources, list_sources_unlocked, Source, sources_path);
    scoped_reader!(
        list_requirements,
        list_requirements_unlocked,
        Requirement,
        requirements_path
    );
    scoped_reader!(list_domains, list_domains_unlocked, Domain, domains_path);
    scoped_reader!(
        list_boundaries,
        list_boundaries_unlocked,
        Boundary,
        boundaries_path
    );
    scoped_reader!(list_topics, list_topics_unlocked, Topic, topics_path);
    scoped_reader!(
        list_questions,
        list_questions_unlocked,
        Question,
        questions_path
    );
    scoped_reader!(
        list_resolutions,
        list_resolutions_unlocked,
        Resolution,
        resolutions_path
    );
    scoped_reader!(list_rules, list_rules_unlocked, Rule, rules_path);
    scoped_reader!(
        list_services,
        list_services_unlocked,
        Service,
        services_path
    );
    scoped_reader!(
        list_service_bindings,
        list_service_bindings_unlocked,
        ServiceBinding,
        service_bindings_path
    );
    scoped_reader!(list_threads, list_threads_unlocked, Thread, threads_path);
    scoped_reader!(
        list_contributions,
        list_contributions_unlocked,
        Contribution,
        contributions_path
    );
    scoped_reader!(
        list_synthesis_packets,
        list_synthesis_packets_unlocked,
        SynthesisPacket,
        synthesis_packets_path
    );
    scoped_reader!(
        list_proposal_cards,
        list_proposal_cards_unlocked,
        ProposalCard,
        proposal_cards_path
    );
    scoped_reader!(
        list_promotion_decisions,
        list_promotion_decisions_unlocked,
        PromotionDecisionRecord,
        promotion_decisions_path
    );

    pub fn list_edges(&self) -> anyhow::Result<Vec<Edge>> {
        self.read_generation(|| self.list_edges_unlocked())
    }

    pub(crate) fn list_edges_unlocked(&self) -> anyhow::Result<Vec<Edge>> {
        read_edge_shards(self)
    }

    pub fn list_messages(&self, scope: &ScopeId) -> anyhow::Result<Vec<Message>> {
        self.read_generation(|| self.list_messages_unlocked(scope))
    }

    pub(crate) fn list_messages_unlocked(&self, scope: &ScopeId) -> anyhow::Result<Vec<Message>> {
        read_message_shards(self, scope)
    }
}

pub(super) fn read_jsonl<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

fn read_jsonl_shards<T: DeserializeOwned>(
    paths: Vec<Utf8PathBuf>,
    kind: &str,
) -> anyhow::Result<Vec<T>> {
    let mut records = Vec::new();
    for path in paths {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {kind} shard {path}"))?;
        for (index, line) in contents.lines().enumerate() {
            records.push(serde_json::from_str(line).with_context(|| {
                format!("failed to parse {kind} shard {path} line {}", index + 1)
            })?);
        }
    }
    Ok(records)
}

fn read_message_shards(store: &StateStore, scope: &ScopeId) -> anyhow::Result<Vec<Message>> {
    let directory = shards::threads_path(&store.layout, scope)
        .parent()
        .unwrap()
        .to_path_buf();
    read_matching_shards(&directory, "message", |path| {
        let Some(name) = path.file_name() else {
            return false;
        };
        let bytes = name.as_bytes();
        bytes.len() == "2026-07.jsonl".len()
            && bytes[0..4].iter().all(u8::is_ascii_digit)
            && bytes[4] == b'-'
            && bytes[5..7].iter().all(u8::is_ascii_digit)
            && &bytes[7..] == b".jsonl"
    })
}

fn read_edge_shards(store: &StateStore) -> anyhow::Result<Vec<Edge>> {
    read_matching_shards(&store.layout.edges_dir(), "edge", |path| {
        path.extension() == Some("jsonl")
    })
}

fn read_matching_shards<T: DeserializeOwned>(
    directory: &Utf8Path,
    kind: &str,
    matches: impl Fn(&Utf8Path) -> bool,
) -> anyhow::Result<Vec<T>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                anyhow::anyhow!("non-UTF-8 {kind} shard path: {}", path.display())
            })?;
            if matches(&path) {
                paths.push(path);
            }
        }
    }
    paths.sort();
    read_jsonl_shards(paths, kind)
}
