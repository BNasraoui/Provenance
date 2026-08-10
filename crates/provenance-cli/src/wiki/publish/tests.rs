use super::{
    publish, publish_with, replace_output_with, write_page, PublicationOutput, PublishError,
    TransactionPaths, OWNERSHIP_MANIFEST,
};
use crate::wiki::model::{
    CorpusCounts, DecisionIndexPage, DomainIndexPage, OrphanReport, PageId, RecordKind,
    RequirementPage, ScopeIndexPage, SearchIndexPage, UnfinishedPage, WikiCorpus,
};
use camino::Utf8PathBuf;
use provenance_core::RequirementStatus;

mod ownership_output_validation;
#[cfg(unix)]
mod preflight_domain;
mod replacement_rollback;
mod route_safety;
mod route_staging;
mod staging_artifact_safety;

fn empty_corpus() -> WikiCorpus {
    WikiCorpus {
        scope: "default".to_string(),
        index: ScopeIndexPage {
            scope: "default".to_string(),
            title: "Provenance Wiki".to_string(),
            counts: CorpusCounts::default(),
            search_coverage: "Search covers requirements, decisions, rules, and sources."
                .to_string(),
            search_example: None,
            domains: Vec::new(),
            authored_domain_count: 0,
            unfinished_count: 0,
        },
        domains: DomainIndexPage {
            scope: "default".to_string(),
            title: "Requirements and rules by domain".to_string(),
            authored_group_count: 0,
            groups: Vec::new(),
            all_requirements: Vec::new(),
            all_rules: Vec::new(),
        },
        search: SearchIndexPage {
            scope: "default".to_string(),
            title: "Search project records".to_string(),
            coverage: "Search covers requirements, decisions, rules, and sources.".to_string(),
            example: None,
            entries: Vec::new(),
        },
        decisions: DecisionIndexPage {
            scope: "default".to_string(),
            title: "Decisions".to_string(),
            entries: Vec::new(),
        },
        unfinished: UnfinishedPage {
            scope: "default".to_string(),
            title: "Unfinished".to_string(),
            gaps: Vec::new(),
            orphans: OrphanReport::default(),
            open_questions: Vec::new(),
        },
        requirements: Vec::new(),
        resolutions: Vec::new(),
        rules: Vec::new(),
        sources: Vec::new(),
    }
}

fn assert_no_transaction_artifacts(output: &camino::Utf8Path) {
    for role in ["lock", "lock.cleanup", "stage", "stage.cleanup", "backup"] {
        assert!(!artifact(output, role).exists());
    }
}

fn artifact(output: &camino::Utf8Path, role: &str) -> Utf8PathBuf {
    let parent = output.parent().unwrap();
    let leaf = output.file_name().unwrap();
    parent.join(format!(".{leaf}.provenance-wiki.{role}"))
}

fn utf8(path: std::path::PathBuf) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(path).unwrap()
}
