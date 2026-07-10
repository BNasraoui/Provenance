use super::ScopeRecords;
use crate::handlers::check::index::CheckIndex;
use crate::handlers::check::references::{
    check_artifact_links, check_origin_references, check_scoped_reference,
};
use provenance_core::ScopeId;

pub(in crate::handlers::check) fn validate_sources_and_requirements(
    records: &ScopeRecords,
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

pub(in crate::handlers::check) fn validate_shaping(
    records: &ScopeRecords,
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

pub(in crate::handlers::check) fn validate_decisions(
    records: &ScopeRecords,
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
