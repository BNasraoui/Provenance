use super::StateStore;
use provenance_core::{CanonicalArtifact, CanonicalArtifactType, ScopeId, StableId};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash)]
struct CanonicalArtifactKey {
    artifact_type: &'static str,
    artifact_id: String,
}

pub(super) struct CanonicalArtifactIndex {
    scope_id: ScopeId,
    entries: HashSet<CanonicalArtifactKey>,
}

impl CanonicalArtifactIndex {
    fn load(store: &StateStore, scope_id: &ScopeId) -> anyhow::Result<Self> {
        let mut entries = HashSet::new();
        extend_scoped(
            &mut entries,
            scope_id,
            CanonicalArtifactType::Source,
            store.list_sources(scope_id)?,
            |record| (record.scope_id, record.id),
        );
        extend_scoped(
            &mut entries,
            scope_id,
            CanonicalArtifactType::Requirement,
            store.list_requirements(scope_id)?,
            |record| (record.scope_id, record.id),
        );
        extend_scoped(
            &mut entries,
            scope_id,
            CanonicalArtifactType::Resolution,
            store.list_resolutions(scope_id)?,
            |record| (record.scope_id, record.id),
        );
        extend_scoped(
            &mut entries,
            scope_id,
            CanonicalArtifactType::Rule,
            store.list_rules(scope_id)?,
            |record| (record.scope_id, record.id),
        );
        Ok(Self {
            scope_id: scope_id.clone(),
            entries,
        })
    }

    pub(super) fn ensure_exists(&self, artifact: Option<&CanonicalArtifact>) -> anyhow::Result<()> {
        let Some(artifact) = artifact else {
            return Ok(());
        };
        anyhow::ensure!(
            self.entries
                .contains(&key(artifact.artifact_type, &artifact.artifact_id)),
            "canonical artifact does not exist in scope {} with kind {:?}: {}",
            self.scope_id.as_str(),
            artifact.artifact_type,
            artifact.artifact_id.as_str()
        );
        Ok(())
    }
}

impl StateStore {
    pub(super) fn canonical_artifact_index(
        &self,
        scope_id: &ScopeId,
    ) -> anyhow::Result<CanonicalArtifactIndex> {
        CanonicalArtifactIndex::load(self, scope_id)
    }
}

fn key(artifact_type: CanonicalArtifactType, artifact_id: &StableId) -> CanonicalArtifactKey {
    let artifact_type = match artifact_type {
        CanonicalArtifactType::Source => "source",
        CanonicalArtifactType::Requirement => "requirement",
        CanonicalArtifactType::Resolution => "resolution",
        CanonicalArtifactType::Rule => "rule",
    };
    CanonicalArtifactKey {
        artifact_type,
        artifact_id: artifact_id.as_str().to_owned(),
    }
}

fn extend_scoped<T>(
    entries: &mut HashSet<CanonicalArtifactKey>,
    scope_id: &ScopeId,
    artifact_type: CanonicalArtifactType,
    records: Vec<T>,
    fields: impl Fn(T) -> (ScopeId, StableId),
) {
    entries.extend(records.into_iter().filter_map(|record| {
        let (embedded_scope_id, artifact_id) = fields(record);
        (embedded_scope_id == *scope_id).then(|| key(artifact_type, &artifact_id))
    }));
}
