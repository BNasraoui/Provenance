use super::{CreateBoundaryInput, CreateQuestionInput, CreateTopicInput, StateStore};
use crate::{jsonl, shards};
use provenance_core::{
    ArtifactLink, ArtifactLinkTargetType, Boundary, Question, SchemaVersion, ScopeId, Topic,
};

impl StateStore {
    pub fn create_boundary(&self, input: CreateBoundaryInput) -> anyhow::Result<Boundary> {
        let CreateBoundaryInput {
            scope_id,
            id,
            requirement_id,
            statement,
            source_ref,
        } = input;
        anyhow::ensure!(
            self.list_requirements(&scope_id)?
                .iter()
                .any(|requirement| requirement.id == requirement_id),
            "requirement does not exist"
        );
        if let Some(source_ref) = &source_ref {
            anyhow::ensure!(
                self.list_sources(&scope_id)?
                    .iter()
                    .any(|source| source.id == source_ref.source_id),
                "source does not exist"
            );
        }
        let mut records = self.list_boundaries(&scope_id)?;
        let boundary = Boundary {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            requirement_id,
            statement,
            source_ref,
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == boundary.id),
            "boundary already exists"
        );
        records.push(boundary.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(&shards::boundaries_path(&self.layout, &scope_id), &records)?;
        Ok(boundary)
    }

    pub fn create_topic(&self, input: CreateTopicInput) -> anyhow::Result<Topic> {
        let CreateTopicInput {
            scope_id,
            id,
            requirement_id,
            title,
            status,
            mut links,
        } = input;
        anyhow::ensure!(
            self.list_requirements(&scope_id)?
                .iter()
                .any(|requirement| requirement.id == requirement_id),
            "requirement does not exist"
        );
        self.validate_artifact_links(&scope_id, &links)?;
        sort_artifact_links(&mut links);
        let mut records = self.list_topics(&scope_id)?;
        let topic = Topic {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            requirement_id,
            title,
            status,
            links,
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == topic.id),
            "topic already exists"
        );
        records.push(topic.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(&shards::topics_path(&self.layout, &scope_id), &records)?;
        Ok(topic)
    }

    pub fn create_question(&self, input: CreateQuestionInput) -> anyhow::Result<Question> {
        let CreateQuestionInput {
            scope_id,
            id,
            topic_id,
            question,
            status,
            answer,
            mut links,
            resolution_id,
        } = input;
        let topic = self
            .list_topics(&scope_id)?
            .into_iter()
            .find(|topic| topic.id == topic_id)
            .ok_or_else(|| anyhow::anyhow!("topic does not exist"))?;
        if let Some(resolution_id) = &resolution_id {
            anyhow::ensure!(
                self.list_resolutions(&scope_id)?
                    .iter()
                    .any(|resolution| &resolution.id == resolution_id),
                "resolution does not exist"
            );
        }
        self.validate_artifact_links(&scope_id, &links)?;
        sort_artifact_links(&mut links);
        let mut records = self.list_questions(&scope_id)?;
        let question = Question {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            topic_id,
            requirement_id: topic.requirement_id,
            question,
            status,
            answer,
            links,
            resolution_id,
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == question.id),
            "question already exists"
        );
        records.push(question.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(&shards::questions_path(&self.layout, &scope_id), &records)?;
        Ok(question)
    }

    fn validate_artifact_links(
        &self,
        scope_id: &ScopeId,
        links: &[ArtifactLink],
    ) -> anyhow::Result<()> {
        for link in links {
            let exists = match link.target_type {
                ArtifactLinkTargetType::Source => self
                    .list_sources(scope_id)?
                    .iter()
                    .any(|source| source.id == link.target_id),
                ArtifactLinkTargetType::Requirement => self
                    .list_requirements(scope_id)?
                    .iter()
                    .any(|requirement| requirement.id == link.target_id),
                ArtifactLinkTargetType::Resolution => self
                    .list_resolutions(scope_id)?
                    .iter()
                    .any(|resolution| resolution.id == link.target_id),
                ArtifactLinkTargetType::Rule => self
                    .list_rules(scope_id)?
                    .iter()
                    .any(|rule| rule.id == link.target_id),
            };
            anyhow::ensure!(exists, "linked artifact does not exist");
        }
        Ok(())
    }
}

fn sort_artifact_links(links: &mut Vec<ArtifactLink>) {
    links.sort_by(|a, b| {
        artifact_link_target_key(a.target_type)
            .cmp(artifact_link_target_key(b.target_type))
            .then(a.target_id.as_str().cmp(b.target_id.as_str()))
    });
    links.dedup();
}

const fn artifact_link_target_key(target_type: ArtifactLinkTargetType) -> &'static str {
    match target_type {
        ArtifactLinkTargetType::Source => "source",
        ArtifactLinkTargetType::Requirement => "requirement",
        ArtifactLinkTargetType::Resolution => "resolution",
        ArtifactLinkTargetType::Rule => "rule",
    }
}
