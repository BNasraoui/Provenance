use super::common::parse_json_arg;
use crate::cli::shaping::TopicsCommand;
use crate::output;
use provenance_core::{ArtifactLink, ScopeId, StableId, TopicStatus};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateTopicInput, ProposalDemand, StateStore, SurfacedProposal},
};

#[derive(serde::Serialize)]
struct TopicClaimResult {
    #[serde(flatten)]
    topic: provenance_core::Topic,
    surfaced_proposals: Vec<SurfacedProposal>,
}

pub(super) fn handle(command: TopicsCommand) -> anyhow::Result<()> {
    match command {
        TopicsCommand::Create {
            repo,
            scope,
            id,
            requirement_id,
            title,
            status,
            links_json,
            format,
        } => {
            let topic =
                StateStore::new(ProvenanceLayout::new(repo)).create_topic(CreateTopicInput {
                    scope_id: ScopeId::new(scope)?,
                    id: StableId::new(id)?,
                    requirement_id: StableId::new(requirement_id)?,
                    title,
                    status: TopicStatus::parse(&status)?,
                    links: parse_json_arg::<Vec<ArtifactLink>>("links-json", &links_json)?,
                })?;
            output::print(format, &topic)?;
        }
        TopicsCommand::List {
            repo,
            scope,
            format,
        } => {
            let topics =
                StateStore::new(ProvenanceLayout::new(repo)).list_topics(&ScopeId::new(scope)?)?;
            output::print(format, &topics)?;
        }
        TopicsCommand::Claim {
            repo,
            scope,
            id,
            actor,
            format,
        } => {
            let store = StateStore::new(ProvenanceLayout::new(repo));
            let scope = ScopeId::new(scope)?;
            let topic_id = StableId::new(id)?;
            let current_topic = store
                .list_topics(&scope)?
                .into_iter()
                .find(|topic| topic.id == topic_id)
                .ok_or_else(|| anyhow::anyhow!("topic does not exist"))?;
            let surfaced_proposals =
                store.surface_proposals(&scope, &ProposalDemand::for_topic(&current_topic))?;
            let topic = store.claim_topic(&scope, &topic_id, &actor)?;
            output::print(
                format,
                &TopicClaimResult {
                    topic,
                    surfaced_proposals,
                },
            )?;
        }
        TopicsCommand::Release {
            repo,
            scope,
            id,
            format,
        } => {
            let topic = StateStore::new(ProvenanceLayout::new(repo))
                .release_topic(&ScopeId::new(scope)?, &StableId::new(id)?)?;
            output::print(format, &topic)?;
        }
        TopicsCommand::Close {
            repo,
            scope,
            id,
            format,
        } => {
            let topic = StateStore::new(ProvenanceLayout::new(repo))
                .close_topic(&ScopeId::new(scope)?, &StableId::new(id)?)?;
            output::print(format, &topic)?;
        }
    }
    Ok(())
}
