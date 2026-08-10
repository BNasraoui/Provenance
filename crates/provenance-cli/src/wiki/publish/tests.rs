use super::{
    publish, publish_with, replace_output_with, write_page, PublicationOutput, PublishError,
    TransactionPaths, OWNERSHIP_MANIFEST,
};
use crate::wiki::model::{
    CorpusCounts, DomainIndexPage, FindingsPage, OrphanReport, PageId, RecordKind, RequirementPage,
    ScopeIndexPage, SearchIndexPage, WikiCorpus,
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
            roots: Vec::new(),
            gaps: Vec::new(),
            orphans: OrphanReport::default(),
            search_coverage: "Search covers requirements and rules.".to_string(),
            domains: Vec::new(),
            authored_domain_count: 0,
            finding_count: 0,
        },
        domains: DomainIndexPage {
            scope: "default".to_string(),
            title: "Requirements and rules by domain".to_string(),
            groups: Vec::new(),
        },
        search: SearchIndexPage {
            scope: "default".to_string(),
            title: "Search requirements and rules".to_string(),
            coverage: "Search covers requirements and rules.".to_string(),
            entries: Vec::new(),
        },
        findings: FindingsPage {
            scope: "default".to_string(),
            title: "Missing evidence".to_string(),
            findings: Vec::new(),
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
