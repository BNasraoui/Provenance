//! The write gate for merged shards.
//!
//! `merge_records` decides which record survives while treating every record as
//! opaque JSON, so a merge writes records that were never checked here: they
//! come from another branch, an older writer, or a hand edit, and the merge is
//! the moment they enter this repository's shard. A per-record check like the
//! edge endpoint table can only fail on a record some side already held, but
//! holding it is exactly what a merge would launder into canonical state.
//! A check spanning the whole file can fail on the merge's own doing; the one
//! such check that exists today, the duplicate id, is refused by `index_by_id`
//! inside the merge itself.
//!
//! So before a merged shard is written, its records go back through the check a
//! direct write would face, and a record that fails it stops the merge instead
//! of landing in the state directory.

use anyhow::Context;
use camino::Utf8Path;
use provenance_core::{edge_validation::validate_edge_endpoint, Edge};
use serde_json::Value;

use super::CanonicalRecord;

/// The record type a shard holds, read off the repository path of the file
/// being merged.
///
/// The path is the only thing the merge driver knows about the file: git hands
/// the driver three temporary files, so only the merged-result path (`%P`)
/// names the shard. Every state shard sits in a directory named after its
/// family, which is what the recognizers below match on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFamily {
    /// `.provenance/state/edges/*.jsonl`
    Edges,
    /// Any other path, including the per-scope record families
    /// (`requirements`, `rules`, `sources`, `resolutions`, and the rest) and
    /// files outside the state directory. Merged records pass unchecked.
    Unrecognized,
}

impl ShardFamily {
    /// Recognizes the family from the path the merged result will be stored at.
    ///
    /// Only the last two directory names are read, so an absolute path, a
    /// repository-relative path, and a path inside a test fixture all resolve
    /// the same way.
    #[must_use]
    pub fn for_shard_path(path: &Utf8Path) -> Self {
        let Some(directory) = path.parent() else {
            return Self::Unrecognized;
        };
        let in_state = directory.parent().and_then(Utf8Path::file_name) == Some("state");
        if in_state && directory.file_name() == Some("edges") {
            Self::Edges
        } else {
            Self::Unrecognized
        }
    }
}

/// Re-checks merged records against the type their shard holds, naming the
/// first record that fails.
///
/// Scope, stated plainly: only the edges shard is checked today, and only for
/// the endpoint table. The per-scope families (requirements, rules, sources,
/// resolutions, boundaries, topics, questions, services, bindings) still merge
/// unchecked; giving them the same treatment means recognizing their directory
/// in [`ShardFamily`] and deserializing each merged record into its own type
/// before applying that type's write-time checks. Cross-record checks that need
/// the whole graph - dangling endpoints, scope membership - belong to
/// `provenance check`, not here: a merge driver sees one file.
pub fn validate_merged_records(
    shard_path: &Utf8Path,
    records: &[CanonicalRecord],
) -> anyhow::Result<()> {
    match ShardFamily::for_shard_path(shard_path) {
        ShardFamily::Edges => validate_merged_edges(records),
        ShardFamily::Unrecognized => Ok(()),
    }
}

fn validate_merged_edges(records: &[CanonicalRecord]) -> anyhow::Result<()> {
    for record in records {
        let named = record
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<record with no id>");
        let edge: Edge = serde_json::from_value(record.clone())
            .with_context(|| format!("merged record {named} is not an edge record"))?;
        validate_edge_endpoint(edge.edge_type, edge.from_type, edge.to_type)
            .with_context(|| format!("merged edge {} is invalid", edge.id.as_str()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(id: &str, edge_type: &str, from_type: &str, to_type: &str) -> Value {
        serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": id,
            "edge_type": edge_type,
            "from_type": from_type,
            "from_id": "node_from",
            "to_type": to_type,
            "to_id": "node_to",
        })
    }

    fn edges_path() -> &'static Utf8Path {
        Utf8Path::new(".provenance/state/edges/edges-00.jsonl")
    }

    #[test]
    fn recognizes_the_edges_shard_by_its_directory() {
        assert_eq!(
            ShardFamily::for_shard_path(edges_path()),
            ShardFamily::Edges
        );
        assert_eq!(
            ShardFamily::for_shard_path(Utf8Path::new(
                "/repo/.provenance/state/edges/edges-01.jsonl"
            )),
            ShardFamily::Edges
        );
    }

    #[test]
    fn leaves_unrecognized_paths_unchecked() {
        for path in [
            ".provenance/state/scopes/default/requirements/req.jsonl",
            "edges/edges-00.jsonl",
            "notes.jsonl",
        ] {
            assert_eq!(
                ShardFamily::for_shard_path(Utf8Path::new(path)),
                ShardFamily::Unrecognized,
                "{path} should not be recognized as a typed shard"
            );
        }
        // A record that would fail edge validation passes when the path does
        // not say the file holds edges.
        validate_merged_records(
            Utf8Path::new("notes.jsonl"),
            &[edge("edge_bad", "references", "rule", "requirement")],
        )
        .unwrap();
    }

    #[test]
    fn accepts_merged_edges_the_endpoint_table_allows() {
        validate_merged_records(
            edges_path(),
            &[
                edge("edge_ok", "references", "source", "requirement"),
                edge("edge_also_ok", "produces", "resolution", "rule"),
            ],
        )
        .unwrap();
    }

    #[test]
    fn rejects_a_merged_edge_the_endpoint_table_forbids() {
        let error = validate_merged_records(
            edges_path(),
            &[
                edge("edge_ok", "references", "source", "requirement"),
                edge("edge_leaves_a_rule", "references", "rule", "requirement"),
            ],
        )
        .unwrap_err();

        let report = format!("{error:#}");
        assert!(
            report.contains("edge_leaves_a_rule"),
            "error should name the offending edge: {report}"
        );
        assert!(
            !report.contains("edge_ok"),
            "error should name only the offending edge: {report}"
        );
    }

    #[test]
    fn rejects_a_merged_record_that_is_not_an_edge() {
        let error = validate_merged_records(
            edges_path(),
            &[serde_json::json!({ "id": "edge_truncated", "edge_type": "references" })],
        )
        .unwrap_err();

        let report = format!("{error:#}");
        assert!(
            report.contains("edge_truncated"),
            "error should name the offending record: {report}"
        );
    }
}
