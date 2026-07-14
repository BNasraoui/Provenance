use camino::Utf8PathBuf;
use provenance_core::ScopeId;

use crate::layout::ProvenanceLayout;

pub fn sources_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("sources/source.jsonl")
}

pub fn requirements_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("requirements/req.jsonl")
}

pub fn domains_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("domains/domain.jsonl")
}

pub fn boundaries_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("boundaries/boundary.jsonl")
}

pub fn topics_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("topics/topic.jsonl")
}

pub fn questions_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("questions/question.jsonl")
}

pub fn resolutions_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("resolutions/res.jsonl")
}

pub fn rules_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("rules/rule.jsonl")
}

pub fn services_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("services/service.jsonl")
}

pub fn service_bindings_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("services/service_binding.jsonl")
}

pub fn threads_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("threads/threads.jsonl")
}

pub fn messages_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("threads/2026-07.jsonl")
}

pub fn contributions_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("ideation/contributions.jsonl")
}

pub fn synthesis_packets_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("ideation/synthesis_packets.jsonl")
}

pub fn proposal_cards_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("ideation/proposal_cards.jsonl")
}

pub fn assertion_records_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("ideation/assertion_records.jsonl")
}

pub fn ideation_landings_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("ideation/landings.jsonl")
}

pub fn promotion_decisions_path(layout: &ProvenanceLayout, scope: &ScopeId) -> Utf8PathBuf {
    layout
        .scopes_dir()
        .join(scope.as_str())
        .join("ideation/promotion_decisions.jsonl")
}

pub fn edges_path(layout: &ProvenanceLayout) -> Utf8PathBuf {
    layout.edges_dir().join("edges-00.jsonl")
}
