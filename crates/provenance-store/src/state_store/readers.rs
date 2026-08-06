use super::{DispositionRecord, Edge, Message, ProvenanceLayout, ScopeId};
use crate::shards;
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use serde::de::DeserializeOwned;

pub(super) fn read_jsonl<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    crate::publication::with_state_path_access(path, || read_jsonl_unlocked(path))
}

fn read_jsonl_unlocked<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

pub(super) fn read_legacy_dispositions(path: &Utf8Path) -> anyhow::Result<Vec<DispositionRecord>> {
    crate::publication::with_state_path_access(path, || read_legacy_dispositions_unlocked(path))
}

fn read_legacy_dispositions_unlocked(path: &Utf8Path) -> anyhow::Result<Vec<DispositionRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| {
            let mut value: serde_json::Value = serde_json::from_str(line)?;
            normalize_disposition_aliases(&mut value);
            Ok(serde_json::from_value(value)?)
        })
        .collect()
}

fn normalize_disposition_aliases(value: &mut serde_json::Value) {
    let Some(record) = value.as_object_mut() else {
        return;
    };
    rename_key(record, "promotionDecisionId", "id");
    rename_key(record, "proposalId", "proposal_id");
    rename_key(record, "decidedBy", "actor");
    rename_key(record, "canonicalArtifact", "canonical_artifact");
    if let Some(actor) = record
        .get_mut("actor")
        .and_then(serde_json::Value::as_object_mut)
    {
        rename_key(actor, "identityType", "identity_type");
        rename_key(actor, "userId", "id");
    }
    if let Some(artifact) = record
        .get_mut("canonical_artifact")
        .and_then(serde_json::Value::as_object_mut)
    {
        rename_key(artifact, "artifactType", "artifact_type");
        rename_key(artifact, "artifactId", "artifact_id");
    }
}

fn rename_key(object: &mut serde_json::Map<String, serde_json::Value>, old: &str, new: &str) {
    if !object.contains_key(new) {
        if let Some(value) = object.remove(old) {
            object.insert(new.to_owned(), value);
        }
    }
}

pub(super) fn read_jsonl_closed<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    crate::publication::with_state_path_access(path, || read_jsonl_closed_unlocked(path))
}

fn read_jsonl_closed_unlocked<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(deserialize_closed)
        .collect()
}

pub(super) fn deserialize_closed<T: DeserializeOwned>(input: &str) -> anyhow::Result<T> {
    let mut unknown = None;
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let value = serde_ignored::deserialize(&mut deserializer, |path| {
        if unknown.is_none() {
            unknown = Some(path.to_string());
        }
    })?;
    if let Some(path) = unknown {
        anyhow::bail!("unknown field `{path}`");
    }
    Ok(value)
}

fn read_jsonl_shards<T: DeserializeOwned>(
    shard_paths: Vec<Utf8PathBuf>,
    shard_kind: &str,
) -> anyhow::Result<Vec<T>> {
    let mut records = Vec::new();
    for path in shard_paths {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {shard_kind} shard {}", path.as_str()))?;
        for (index, line) in contents.lines().enumerate() {
            records.push(serde_json::from_str(line).with_context(|| {
                format!(
                    "failed to parse {shard_kind} shard {} line {}",
                    path.as_str(),
                    index + 1
                )
            })?);
        }
    }
    Ok(records)
}

pub(super) fn read_message_shards(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
) -> anyhow::Result<Vec<Message>> {
    crate::publication::with_repository_publication(layout, || {
        let threads_dir = shards::threads_path(layout, scope)
            .parent()
            .expect("threads path must have a parent")
            .to_path_buf();
        if !threads_dir.exists() {
            return Ok(Vec::new());
        }
        let mut shard_paths = Vec::new();
        for entry in std::fs::read_dir(&threads_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                    anyhow::anyhow!("non-UTF-8 message shard path: {}", path.display())
                })?;
                if is_message_month_shard(&path) {
                    shard_paths.push(path);
                }
            }
        }
        shard_paths.sort();
        read_jsonl_shards(shard_paths, "message")
    })
}

fn is_message_month_shard(path: &Utf8Path) -> bool {
    let Some(file_name) = path.file_name() else {
        return false;
    };
    let bytes = file_name.as_bytes();
    bytes.len() == "2026-07.jsonl".len()
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && &bytes[7..] == b".jsonl"
}

pub(super) fn read_edge_shards(
    layout: &ProvenanceLayout,
    closed_scope: Option<&ScopeId>,
) -> anyhow::Result<Vec<Edge>> {
    crate::publication::with_repository_publication(layout, || {
        let edges_dir = layout.edges_dir();
        if !edges_dir.exists() {
            return Ok(Vec::new());
        }
        let mut shard_paths = Vec::new();
        for entry in std::fs::read_dir(&edges_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                    anyhow::anyhow!("non-UTF-8 edge shard path: {}", path.display())
                })?;
                if path.extension() == Some("jsonl") {
                    shard_paths.push(path);
                }
            }
        }
        shard_paths.sort();
        if let Some(scope) = closed_scope {
            read_closed_edges(shard_paths, scope)
        } else {
            read_jsonl_shards(shard_paths, "edge")
        }
    })
}

fn read_closed_edges(paths: Vec<Utf8PathBuf>, scope: &ScopeId) -> anyhow::Result<Vec<Edge>> {
    let mut records = Vec::new();
    for path in paths {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read edge shard {}", path.as_str()))?;
        for (index, line) in contents.lines().enumerate() {
            let value: serde_json::Value = serde_json::from_str(line).with_context(|| {
                format!(
                    "failed to parse edge shard {} line {}",
                    path.as_str(),
                    index + 1
                )
            })?;
            if value.get("scope_id").and_then(serde_json::Value::as_str) == Some(scope.as_str()) {
                records.push(deserialize_closed(line).with_context(|| {
                    format!(
                        "failed to parse edge shard {} line {}",
                        path.as_str(),
                        index + 1
                    )
                })?);
            }
        }
    }
    Ok(records)
}
