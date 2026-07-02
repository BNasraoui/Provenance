use super::{
    CreateBoundaryInput, CreateQuestionInput, CreateTopicInput, StateStore, UpdateQuestionInput,
};
use crate::{jsonl, shards};
use provenance_core::{
    ArtifactLink, ArtifactLinkTargetType, Boundary, Question, QuestionStatus, SchemaVersion,
    ScopeId, StableId, Topic, TopicStatus,
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
            claimed_by: None,
            claimed_at: None,
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
            resolution_method,
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
            resolution_method,
            status,
            claimed_by: None,
            claimed_at: None,
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

    pub fn claim_topic(
        &self,
        scope_id: &ScopeId,
        id: &StableId,
        actor: &str,
    ) -> anyhow::Result<Topic> {
        let actor = validated_actor(actor)?;
        let claimed_at = now_ms()?;
        self.update_topic(scope_id, id, |topic| {
            anyhow::ensure!(
                topic.status != TopicStatus::Closed,
                "topic {} is closed and cannot be claimed",
                topic.id.as_str()
            );
            if let Some(holder) = &topic.claimed_by {
                anyhow::bail!("topic {} is already claimed by {holder}", topic.id.as_str());
            }
            topic.claimed_by = Some(actor);
            topic.claimed_at = Some(claimed_at);
            Ok(())
        })
    }

    pub fn release_topic(&self, scope_id: &ScopeId, id: &StableId) -> anyhow::Result<Topic> {
        self.update_topic(scope_id, id, |topic| {
            anyhow::ensure!(
                topic.claimed_by.is_some(),
                "topic {} is not claimed",
                topic.id.as_str()
            );
            topic.claimed_by = None;
            topic.claimed_at = None;
            Ok(())
        })
    }

    pub fn close_topic(&self, scope_id: &ScopeId, id: &StableId) -> anyhow::Result<Topic> {
        self.update_topic(scope_id, id, |topic| {
            topic.status = TopicStatus::Closed;
            topic.claimed_by = None;
            topic.claimed_at = None;
            Ok(())
        })
    }

    pub fn claim_question(
        &self,
        scope_id: &ScopeId,
        id: &StableId,
        actor: &str,
    ) -> anyhow::Result<Question> {
        let actor = validated_actor(actor)?;
        let claimed_at = now_ms()?;
        self.mutate_question(scope_id, id, |question| {
            match question.status {
                QuestionStatus::Open => {}
                QuestionStatus::BlockedOnHuman => anyhow::bail!(
                    "question {} is blocked_on_human and cannot be claimed",
                    question.id.as_str()
                ),
                QuestionStatus::Answered => anyhow::bail!(
                    "question {} is answered and cannot be claimed",
                    question.id.as_str()
                ),
            }
            if let Some(holder) = &question.claimed_by {
                anyhow::bail!(
                    "question {} is already claimed by {holder}",
                    question.id.as_str()
                );
            }
            question.claimed_by = Some(actor);
            question.claimed_at = Some(claimed_at);
            Ok(())
        })
    }

    pub fn release_question(&self, scope_id: &ScopeId, id: &StableId) -> anyhow::Result<Question> {
        self.mutate_question(scope_id, id, |question| {
            anyhow::ensure!(
                question.claimed_by.is_some(),
                "question {} is not claimed",
                question.id.as_str()
            );
            question.claimed_by = None;
            question.claimed_at = None;
            Ok(())
        })
    }

    pub fn answer_question(
        &self,
        scope_id: &ScopeId,
        id: &StableId,
        answer: String,
        resolution_id: Option<StableId>,
    ) -> anyhow::Result<Question> {
        anyhow::ensure!(!answer.trim().is_empty(), "answer must not be empty");
        if let Some(resolution_id) = &resolution_id {
            anyhow::ensure!(
                self.list_resolutions(scope_id)?
                    .iter()
                    .any(|resolution| &resolution.id == resolution_id),
                "resolution does not exist"
            );
        }
        self.mutate_question(scope_id, id, |question| {
            question.status = QuestionStatus::Answered;
            question.answer = Some(answer);
            if resolution_id.is_some() {
                question.resolution_id = resolution_id;
            }
            question.claimed_by = None;
            question.claimed_at = None;
            Ok(())
        })
    }

    pub fn update_question(&self, input: UpdateQuestionInput) -> anyhow::Result<Question> {
        let UpdateQuestionInput {
            scope_id,
            id,
            resolution_method,
            status,
            mut links,
            resolution_id,
        } = input;
        anyhow::ensure!(
            resolution_method.is_some()
                || status.is_some()
                || links.is_some()
                || resolution_id.is_some(),
            "at least one question field must be updated"
        );
        if let Some(resolution_id) = &resolution_id {
            anyhow::ensure!(
                self.list_resolutions(&scope_id)?
                    .iter()
                    .any(|resolution| &resolution.id == resolution_id),
                "resolution does not exist"
            );
        }
        if let Some(links) = &mut links {
            self.validate_artifact_links(&scope_id, links)?;
            sort_artifact_links(links);
        }
        self.mutate_question(&scope_id, &id, |question| {
            if let Some(resolution_method) = resolution_method {
                question.resolution_method = resolution_method;
            }
            if let Some(status) = status {
                anyhow::ensure!(
                    status != QuestionStatus::Answered || question.answer.is_some(),
                    "use questions answer --answer to answer a question"
                );
                question.status = status;
                if status != QuestionStatus::Open {
                    question.claimed_by = None;
                    question.claimed_at = None;
                }
            }
            if let Some(links) = links {
                question.links = links;
            }
            if let Some(resolution_id) = resolution_id {
                question.resolution_id = Some(resolution_id);
            }
            Ok(())
        })
    }

    fn update_topic(
        &self,
        scope_id: &ScopeId,
        id: &StableId,
        mutate: impl FnOnce(&mut Topic) -> anyhow::Result<()>,
    ) -> anyhow::Result<Topic> {
        let mut records = self.list_topics(scope_id)?;
        let topic = records
            .iter_mut()
            .find(|topic| &topic.id == id)
            .ok_or_else(|| anyhow::anyhow!("topic does not exist"))?;
        mutate(topic)?;
        let updated = topic.clone();
        jsonl::write_jsonl_atomic(&shards::topics_path(&self.layout, scope_id), &records)?;
        Ok(updated)
    }

    fn mutate_question(
        &self,
        scope_id: &ScopeId,
        id: &StableId,
        mutate: impl FnOnce(&mut Question) -> anyhow::Result<()>,
    ) -> anyhow::Result<Question> {
        let mut records = self.list_questions(scope_id)?;
        let question = records
            .iter_mut()
            .find(|question| &question.id == id)
            .ok_or_else(|| anyhow::anyhow!("question does not exist"))?;
        mutate(question)?;
        let updated = question.clone();
        jsonl::write_jsonl_atomic(&shards::questions_path(&self.layout, scope_id), &records)?;
        Ok(updated)
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

fn validated_actor(actor: &str) -> anyhow::Result<String> {
    let actor = actor.trim();
    anyhow::ensure!(!actor.is_empty(), "actor must not be empty");
    Ok(actor.to_string())
}

fn now_ms() -> anyhow::Result<i64> {
    Ok(i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis(),
    )?)
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
