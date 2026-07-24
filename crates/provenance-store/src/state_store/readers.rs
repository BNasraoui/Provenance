use crate::{layout::ProvenanceLayout, shards};
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_core::{Edge, Message, ScopeId};
use serde::de::DeserializeOwned;

pub(super) fn read_jsonl<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

pub(super) fn read_jsonl_closed<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
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

pub(super) fn read_message_shards(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
) -> anyhow::Result<Vec<Message>> {
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
}

pub(super) fn read_edge_shards(
    layout: &ProvenanceLayout,
    closed_scope: Option<&ScopeId>,
) -> anyhow::Result<Vec<Edge>> {
    let edges_dir = layout.edges_dir();
    if !edges_dir.exists() {
        return Ok(Vec::new());
    }

    let mut shard_paths = Vec::new();
    for entry in std::fs::read_dir(&edges_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|path| anyhow::anyhow!("non-UTF-8 edge shard path: {}", path.display()))?;
            if path.extension() == Some("jsonl") {
                shard_paths.push(path);
            }
        }
    }
    shard_paths.sort();

    if let Some(scope) = closed_scope {
        read_closed_scope_edges(shard_paths, scope)
    } else {
        read_jsonl_shards(shard_paths, "edge")
    }
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

fn read_closed_scope_edges(
    shard_paths: Vec<Utf8PathBuf>,
    scope: &ScopeId,
) -> anyhow::Result<Vec<Edge>> {
    let mut records = Vec::new();
    for path in shard_paths {
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
            if value.get("scope_id").and_then(serde_json::Value::as_str) != Some(scope.as_str()) {
                continue;
            }
            records.push(deserialize_closed(line).with_context(|| {
                format!(
                    "failed to parse edge shard {} line {}",
                    path.as_str(),
                    index + 1
                )
            })?);
        }
    }
    Ok(records)
}
