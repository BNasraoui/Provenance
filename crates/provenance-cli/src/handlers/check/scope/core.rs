use crate::handlers::check::index::CheckIndex;
use crate::handlers::check::references::{
    check_artifact_links, check_origin_references, check_scoped_reference,
};
use provenance_core::{
    Boundary, Domain, Question, Requirement, Resolution, Rule, ScopeId, Source, Topic,
};
use provenance_store::state_store::StateStore;

pub(super) struct Records {
    sources: Vec<Source>,
    domains: Vec<Domain>,
    requirements: Vec<Requirement>,
    boundaries: Vec<Boundary>,
    topics: Vec<Topic>,
    questions: Vec<Question>,
    resolutions: Vec<Resolution>,
    rules: Vec<Rule>,
}

impl Records {
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
        })
    }

    pub(super) fn validate_scope_ownership(
        &self,
        loaded_scope_id: &ScopeId,
        findings: &mut Vec<String>,
    ) {
        macro_rules! check_records {
            ($records:expr, $record_type:literal) => {
                for record in $records {
                    super::check_scope_ownership(
                        loaded_scope_id,
                        &record.scope_id,
                        $record_type,
                        &record.id,
                        findings,
                    );
                }
            };
        }

        check_records!(&self.sources, "source");
        check_records!(&self.domains, "domain");
        check_records!(&self.requirements, "requirement");
        check_records!(&self.boundaries, "boundary");
        check_records!(&self.topics, "topic");
        check_records!(&self.questions, "question");
        check_records!(&self.resolutions, "resolution");
        check_records!(&self.rules, "rule");
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
    }

    pub(super) fn validate(
        &self,
        index: &CheckIndex,
        scope_id: &ScopeId,
        dangling: &mut Vec<String>,
    ) {
        validate_sources_and_requirements(self, index, scope_id, dangling);
        validate_shaping(self, index, scope_id, dangling);
        validate_decisions(self, index, scope_id, dangling);
    }
}

fn validate_sources_and_requirements(
    records: &Records,
    index: &CheckIndex,
    scope_id: &ScopeId,
    dangling: &mut Vec<String>,
) {
    for source in &records.sources {
        let owner = format!("source {}", source.id.as_str());
        if let Some(superseded_by) = &source.superseded_by {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &owner,
                "superseded_by",
                "source",
                superseded_by,
            );
        }
        check_origin_references(
            index,
            dangling,
            scope_id,
            &owner,
            source.origin_thread.as_ref(),
            source.origin_message.as_ref(),
        );
    }
    for requirement in &records.requirements {
        let owner = format!("requirement {}", requirement.id.as_str());
        if let Some(domain_id) = &requirement.domain_id {
            check_scoped_reference(
                index, dangling, scope_id, &owner, "domain", "domain", domain_id,
            );
        }
        for source_ref in &requirement.source_refs {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &owner,
                "source_ref",
                "source",
                &source_ref.source_id,
            );
        }
        check_origin_references(
            index,
            dangling,
            scope_id,
            &owner,
            requirement.origin_thread.as_ref(),
            requirement.origin_message.as_ref(),
        );
    }
}

fn validate_shaping(
    records: &Records,
    index: &CheckIndex,
    scope_id: &ScopeId,
    dangling: &mut Vec<String>,
) {
    for boundary in &records.boundaries {
        let owner = format!("boundary {}", boundary.id.as_str());
        check_scoped_reference(
            index,
            dangling,
            scope_id,
            &owner,
            "requirement",
            "requirement",
            &boundary.requirement_id,
        );
        if let Some(source_ref) = &boundary.source_ref {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &owner,
                "source_ref",
                "source",
                &source_ref.source_id,
            );
        }
    }
    for topic in &records.topics {
        let owner = format!("topic {}", topic.id.as_str());
        check_scoped_reference(
            index,
            dangling,
            scope_id,
            &owner,
            "requirement",
            "requirement",
            &topic.requirement_id,
        );
        check_artifact_links(index, dangling, scope_id, &owner, &topic.links);
    }
    for question in &records.questions {
        let owner = format!("question {}", question.id.as_str());
        check_scoped_reference(
            index,
            dangling,
            scope_id,
            &owner,
            "topic",
            "topic",
            &question.topic_id,
        );
        check_scoped_reference(
            index,
            dangling,
            scope_id,
            &owner,
            "requirement",
            "requirement",
            &question.requirement_id,
        );
        if let Some(resolution_id) = &question.resolution_id {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &owner,
                "resolution",
                "resolution",
                resolution_id,
            );
        }
        check_artifact_links(index, dangling, scope_id, &owner, &question.links);
    }
}

fn validate_decisions(
    records: &Records,
    index: &CheckIndex,
    scope_id: &ScopeId,
    dangling: &mut Vec<String>,
) {
    for resolution in &records.resolutions {
        let owner = format!("resolution {}", resolution.id.as_str());
        if let Some(superseded_by) = &resolution.superseded_by {
            check_scoped_reference(
                index,
                dangling,
                scope_id,
                &owner,
                "superseded_by",
                "resolution",
                superseded_by,
            );
        }
        check_origin_references(
            index,
            dangling,
            scope_id,
            &owner,
            resolution.origin_thread.as_ref(),
            resolution.origin_message.as_ref(),
        );
    }
    for rule in &records.rules {
        check_origin_references(
            index,
            dangling,
            scope_id,
            &format!("rule {}", rule.id.as_str()),
            rule.origin_thread.as_ref(),
            rule.origin_message.as_ref(),
        );
    }
}
