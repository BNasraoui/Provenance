use super::index::CheckIndex;
use provenance_core::{
    Boundary, Contribution, Domain, Message, PromotionDecisionRecord, ProposalCard, Question,
    Requirement, Resolution, Rule, ScopeId, Service, ServiceBinding, Source, SynthesisPacket,
    Thread, Topic,
};
use provenance_store::state_store::StateStore;

pub(super) mod collaboration;
pub(super) mod core;
pub(super) mod ideation;

pub(super) struct ScopeRecords {
    sources: Vec<Source>,
    domains: Vec<Domain>,
    requirements: Vec<Requirement>,
    boundaries: Vec<Boundary>,
    topics: Vec<Topic>,
    questions: Vec<Question>,
    resolutions: Vec<Resolution>,
    rules: Vec<Rule>,
    services: Vec<Service>,
    service_bindings: Vec<ServiceBinding>,
    threads: Vec<Thread>,
    messages: Vec<Message>,
    contributions: Vec<Contribution>,
    synthesis_packets: Vec<SynthesisPacket>,
    proposal_cards: Vec<ProposalCard>,
    promotion_decisions: Vec<PromotionDecisionRecord>,
}

impl ScopeRecords {
    pub(super) fn load(store: &StateStore, scope_id: &ScopeId) -> anyhow::Result<Self> {
        Ok(Self {
            sources: store.list_sources(scope_id)?,
            domains: store.list_domains(scope_id)?,
            requirements: store.list_requirements(scope_id)?,
            boundaries: store.list_boundaries(scope_id)?,
            topics: store.list_topics(scope_id)?,
            questions: store.list_questions(scope_id)?,
            resolutions: store.list_resolutions(scope_id)?,
            rules: store.list_rules(scope_id)?,
            services: store.list_services(scope_id)?,
            service_bindings: store.list_service_bindings(scope_id)?,
            threads: store.list_threads(scope_id)?,
            messages: store.list_messages(scope_id)?,
            contributions: store.list_contributions(scope_id)?,
            synthesis_packets: store.list_synthesis_packets(scope_id)?,
            proposal_cards: store.list_proposal_cards(scope_id)?,
            promotion_decisions: store.list_promotion_decisions(scope_id)?,
        })
    }

    pub(super) fn add_to(&self, index: &mut CheckIndex) {
        for source in &self.sources {
            index.add_node(&source.scope_id, "source", &source.id);
        }
        for domain in &self.domains {
            index.add_node(&domain.scope_id, "domain", &domain.id);
        }
        for requirement in &self.requirements {
            index.add_node(&requirement.scope_id, "requirement", &requirement.id);
        }
        for boundary in &self.boundaries {
            index.add_node(&boundary.scope_id, "boundary", &boundary.id);
        }
        for topic in &self.topics {
            index.add_node(&topic.scope_id, "topic", &topic.id);
        }
        for question in &self.questions {
            index.add_node(&question.scope_id, "question", &question.id);
        }
        for resolution in &self.resolutions {
            index.add_node(&resolution.scope_id, "resolution", &resolution.id);
        }
        for rule in &self.rules {
            index.add_node(&rule.scope_id, "rule", &rule.id);
        }
        for service in &self.services {
            index.add_node(&service.scope_id, "service", &service.id);
        }
        for thread in &self.threads {
            index.add_node(&thread.scope_id, "thread", &thread.id);
        }
        for message in &self.messages {
            index.add_node(&message.scope_id, "message", &message.id);
        }
        for proposal in &self.proposal_cards {
            index.add_node(&proposal.scope_id, "proposal", &proposal.id);
        }
    }
}
