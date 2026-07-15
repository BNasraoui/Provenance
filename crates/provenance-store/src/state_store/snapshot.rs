use super::StateStore;
use provenance_core::{
    Boundary, Contribution, Domain, Edge, Manifest, Message, PromotionDecisionRecord, ProposalCard,
    Question, Requirement, Resolution, Rule, ScopeId, Service, ServiceBinding, Source,
    SynthesisPacket, Thread, Topic,
};

/// An owned, immutable view of every record in one scope and generation.
#[derive(Debug)]
pub struct ScopeSnapshot {
    pub scope: ScopeId,
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
    pub proposals: Vec<ProposalCard>,
    pub promotion_decisions: Vec<PromotionDecisionRecord>,
}

#[derive(Debug)]
pub struct RepositorySnapshot {
    pub manifest: Manifest,
    pub scope_directories: Vec<String>,
    pub scopes: Vec<ScopeSnapshot>,
    pub edges: Vec<Edge>,
}

impl StateStore {
    /// Recovers and reads every scope record while holding the generation lock.
    pub fn scope_snapshot(&self, scope: &ScopeId) -> anyhow::Result<ScopeSnapshot> {
        self.read_generation(|| self.scope_snapshot_unlocked(scope, true))
    }

    pub fn repository_snapshot(&self) -> anyhow::Result<RepositorySnapshot> {
        self.read_generation(|| {
            let manifest = self.manifest_unlocked()?;
            let scopes = manifest
                .scopes
                .iter()
                .map(|scope| self.scope_snapshot_unlocked(&scope.id, false))
                .collect::<anyhow::Result<_>>()?;
            let scope_directories = self.list_scope_directories_unlocked()?;
            let edges = self.list_edges_unlocked()?;
            Ok(RepositorySnapshot {
                manifest,
                scope_directories,
                scopes,
                edges,
            })
        })
    }

    fn scope_snapshot_unlocked(
        &self,
        scope: &ScopeId,
        filter_embedded_scope: bool,
    ) -> anyhow::Result<ScopeSnapshot> {
        let mut snapshot = ScopeSnapshot {
            scope: scope.clone(),
            sources: self.list_sources_unlocked(scope)?,
            domains: self.list_domains_unlocked(scope)?,
            requirements: self.list_requirements_unlocked(scope)?,
            boundaries: self.list_boundaries_unlocked(scope)?,
            topics: self.list_topics_unlocked(scope)?,
            questions: self.list_questions_unlocked(scope)?,
            resolutions: self.list_resolutions_unlocked(scope)?,
            rules: self.list_rules_unlocked(scope)?,
            services: self.list_services_unlocked(scope)?,
            service_bindings: self.list_service_bindings_unlocked(scope)?,
            edges: self
                .list_edges_unlocked()?
                .into_iter()
                .filter(|edge| edge.scope_id == *scope)
                .collect(),
            threads: self.list_threads_unlocked(scope)?,
            messages: self.list_messages_unlocked(scope)?,
            contributions: self.list_contributions_unlocked(scope)?,
            synthesis_packets: self.list_synthesis_packets_unlocked(scope)?,
            proposals: self.list_proposal_cards_unlocked(scope)?,
            promotion_decisions: self.list_promotion_decisions_unlocked(scope)?,
        };
        if filter_embedded_scope {
            snapshot.sources.retain(|record| record.scope_id == *scope);
            snapshot.domains.retain(|record| record.scope_id == *scope);
            snapshot
                .requirements
                .retain(|record| record.scope_id == *scope);
            snapshot
                .boundaries
                .retain(|record| record.scope_id == *scope);
            snapshot.topics.retain(|record| record.scope_id == *scope);
            snapshot
                .questions
                .retain(|record| record.scope_id == *scope);
            snapshot
                .resolutions
                .retain(|record| record.scope_id == *scope);
            snapshot.rules.retain(|record| record.scope_id == *scope);
            snapshot.services.retain(|record| record.scope_id == *scope);
            snapshot
                .service_bindings
                .retain(|record| record.scope_id == *scope);
            snapshot.threads.retain(|record| record.scope_id == *scope);
            snapshot.messages.retain(|record| record.scope_id == *scope);
            snapshot
                .contributions
                .retain(|record| record.scope_id == *scope);
            snapshot
                .synthesis_packets
                .retain(|record| record.scope_id == *scope);
            snapshot
                .proposals
                .retain(|record| record.scope_id == *scope);
            snapshot
                .promotion_decisions
                .retain(|record| record.scope_id == *scope);
        }
        Ok(snapshot)
    }
}
