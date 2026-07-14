use super::common::parse_json_arg;
use crate::cli::shaping::TopicsCommand;
use crate::output;
use provenance_core::{ArtifactLink, ScopeId, StableId, TopicStatus};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateTopicInput, StateStore},
};

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
            let topic = StateStore::new(ProvenanceLayout::new(repo)).claim_topic(
                &ScopeId::new(scope)?,
                &StableId::new(id)?,
                &actor,
            )?;
            output::print(format, &topic)?;
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
