use super::StateStore;
use crate::{shards, transaction::StateTransaction};
use camino::Utf8PathBuf;
use provenance_core::{
    validate_record_scope, validate_unique_ids, Boundary, Contribution, Domain, Edge, Message,
    PromotionDecisionRecord, ProposalCard, Question, Requirement, Resolution, Rule, ScopeId,
    Service, ServiceBinding, Source, SynthesisPacket, Thread, Topic,
};

#[derive(Default)]
pub struct ScopeReplacement {
    pub sources: Vec<Source>,
    pub domains: Vec<Domain>,
    pub requirements: Vec<Requirement>,
    pub boundaries: Vec<Boundary>,
    pub topics: Vec<Topic>,
    pub questions: Vec<Question>,
    pub resolutions: Vec<Resolution>,
    pub rules: Vec<Rule>,
    pub services: Vec<Service>,
    pub service_bindings: Vec<ServiceBinding>,
    pub edges: Vec<Edge>,
    pub threads: Vec<Thread>,
    pub messages: Vec<Message>,
    pub contributions: Vec<Contribution>,
    pub synthesis_packets: Vec<SynthesisPacket>,
    pub proposal_cards: Vec<ProposalCard>,
    pub promotion_decisions: Vec<PromotionDecisionRecord>,
}

impl ScopeReplacement {
    pub fn validate_for_scope(&self, scope: &ScopeId) -> anyhow::Result<()> {
        macro_rules! validate_collection {
            ($kind:literal, $records:expr) => {{
                for record in $records {
                    validate_record_scope(scope, &record.scope_id, $kind, &record.id)?;
                }
                validate_unique_ids($kind, $records.iter().map(|record| &record.id))?;
            }};
        }
        validate_collection!("source", &self.sources);
        validate_collection!("domain", &self.domains);
        validate_collection!("requirement", &self.requirements);
        validate_collection!("boundary", &self.boundaries);
        validate_collection!("topic", &self.topics);
        validate_collection!("question", &self.questions);
        validate_collection!("resolution", &self.resolutions);
        validate_collection!("rule", &self.rules);
        validate_collection!("service", &self.services);
        validate_collection!("service binding", &self.service_bindings);
        validate_collection!("edge", &self.edges);
        validate_collection!("thread", &self.threads);
        validate_collection!("message", &self.messages);
        validate_collection!("contribution", &self.contributions);
        validate_collection!("synthesis packet", &self.synthesis_packets);
        validate_collection!("proposal", &self.proposal_cards);
        validate_collection!("promotion decision", &self.promotion_decisions);
        Ok(())
    }
}

impl StateStore {
    /// Replaces one scope using only shard paths derived from this store's layout.
    ///
    /// Raw transactions are intentionally unavailable to external callers.
    /// ```compile_fail
    /// use provenance_store::transaction::StateTransaction;
    /// ```
    pub fn replace_scope(
        &self,
        scope: &ScopeId,
        replacement: &ScopeReplacement,
    ) -> anyhow::Result<()> {
        self.write_transaction(|transaction| {
            replacement.validate_for_scope(scope)?;
            anyhow::ensure!(
                self.manifest_unlocked()?
                    .scopes
                    .iter()
                    .any(|manifest_scope| manifest_scope.id == *scope),
                "scope {} is absent from manifest",
                scope.as_str()
            );
            let edge_paths = self.edge_shard_paths()?;
            let message_paths = self.message_shard_paths(scope)?;
            self.replace_scope_locked(transaction, scope, replacement, &edge_paths, &message_paths)
        })
    }

    fn message_shard_paths(&self, scope: &ScopeId) -> anyhow::Result<Vec<Utf8PathBuf>> {
        let directory = shards::threads_path(&self.layout, scope)
            .parent()
            .unwrap()
            .to_path_buf();
        Self::shard_paths(
            &directory,
            shards::messages_path(&self.layout, scope),
            |path| {
                let Some(name) = path.file_name() else {
                    return false;
                };
                let bytes = name.as_bytes();
                bytes.len() == "2026-07.jsonl".len()
                    && bytes[0..4].iter().all(u8::is_ascii_digit)
                    && bytes[4] == b'-'
                    && bytes[5..7].iter().all(u8::is_ascii_digit)
                    && &bytes[7..] == b".jsonl"
            },
        )
    }

    fn edge_shard_paths(&self) -> anyhow::Result<Vec<Utf8PathBuf>> {
        let directory = self.layout.edges_dir();
        Self::shard_paths(&directory, shards::edges_path(&self.layout), |path| {
            path.extension() == Some("jsonl")
        })
    }

    fn shard_paths(
        directory: &camino::Utf8Path,
        canonical: Utf8PathBuf,
        matches: impl Fn(&camino::Utf8Path) -> bool,
    ) -> anyhow::Result<Vec<Utf8PathBuf>> {
        let mut paths = Vec::new();
        if directory.exists() {
            for entry in std::fs::read_dir(directory)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                        anyhow::anyhow!("non-UTF-8 shard path: {}", path.display())
                    })?;
                    if matches(&path) {
                        paths.push(path);
                    }
                }
            }
        }
        if !paths.contains(&canonical) {
            paths.push(canonical);
        }
        paths.sort();
        Ok(paths)
    }

    fn replace_scope_locked(
        &self,
        transaction: &mut StateTransaction,
        scope: &ScopeId,
        replacement: &ScopeReplacement,
        edge_paths: &[Utf8PathBuf],
        message_paths: &[Utf8PathBuf],
    ) -> anyhow::Result<()> {
        macro_rules! replace {
            ($path:ident, $records:expr) => {
                transaction.replace_jsonl(&shards::$path(&self.layout, scope), $records)?;
            };
        }
        replace!(sources_path, &replacement.sources);
        replace!(domains_path, &replacement.domains);
        replace!(requirements_path, &replacement.requirements);
        replace!(boundaries_path, &replacement.boundaries);
        replace!(topics_path, &replacement.topics);
        replace!(questions_path, &replacement.questions);
        replace!(resolutions_path, &replacement.resolutions);
        replace!(rules_path, &replacement.rules);
        replace!(services_path, &replacement.services);
        replace!(service_bindings_path, &replacement.service_bindings);
        replace!(threads_path, &replacement.threads);
        replace!(contributions_path, &replacement.contributions);
        replace!(synthesis_packets_path, &replacement.synthesis_packets);
        replace!(proposal_cards_path, &replacement.proposal_cards);
        replace!(promotion_decisions_path, &replacement.promotion_decisions);

        let canonical_messages = shards::messages_path(&self.layout, scope);
        for path in message_paths {
            let mut messages = transaction
                .read_jsonl::<Message>(path)?
                .into_iter()
                .filter(|message| message.scope_id != *scope)
                .collect::<Vec<_>>();
            if *path == canonical_messages {
                messages.extend(replacement.messages.iter().cloned());
            }
            messages.sort_by(|a, b| {
                a.scope_id
                    .as_str()
                    .cmp(b.scope_id.as_str())
                    .then(a.id.as_str().cmp(b.id.as_str()))
            });
            transaction.replace_jsonl(path, &messages)?;
        }

        let canonical = shards::edges_path(&self.layout);
        for path in edge_paths {
            let mut edges = transaction
                .read_jsonl::<Edge>(path)?
                .into_iter()
                .filter(|edge| edge.scope_id != *scope)
                .collect::<Vec<_>>();
            if *path == canonical {
                edges.extend(replacement.edges.iter().cloned());
            }
            edges.sort_by(|a, b| {
                a.scope_id
                    .as_str()
                    .cmp(b.scope_id.as_str())
                    .then(a.id.as_str().cmp(b.id.as_str()))
            });
            transaction.replace_jsonl(path, &edges)?;
        }
        Ok(())
    }
}
