use super::{DispositionRecord, Edge, Message, ProvenanceLayout, ScopeId};
use crate::shards;
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_core::SUPPORTED_SCHEMA_VERSION;
use provenance_macros::rule;
use serde::de::DeserializeOwned;

/// What a reader does with fields its structs do not know.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fields {
    /// Unknown fields are ignored: the reader takes what it recognises.
    Open,
    /// Unknown fields are refused, so a pinned graph carries nothing extra.
    Closed,
}

/// The one place a stored line becomes a record.
///
/// Every reader in this module lands here: [`read_records`] carries the open
/// families, the closed families and the aliased legacy dispositions;
/// [`read_jsonl_shards`] carries messages and edges; [`read_closed_edges`]
/// carries the edges a pinned graph keeps. A family that does not pass
/// through this function is not read at all, which is what makes the version
/// guard below cover every record rather than the ones somebody remembered.
///
/// The caller has already turned the line into a [`serde_json::Value`],
/// because some of them look at it first: the closed edge reader drops
/// another scope's rows before anything here judges them.
fn record_from_line<T: DeserializeOwned>(
    path: &Utf8Path,
    line_number: usize,
    line: &str,
    value: serde_json::Value,
    fields: Fields,
) -> anyhow::Result<T> {
    ensure_supported_record_version(path, line_number, &value)?;
    match fields {
        Fields::Open => Ok(serde_json::from_value(value)?),
        // The closed reader needs the untouched line: `serde_ignored` reports
        // an unknown field by walking the document as it was written.
        Fields::Closed => deserialize_closed(line),
    }
}

/// Nothing is loaded from disk except at the schema version this build reads.
///
/// A stored record says which layout it was written in. A later version means
/// a different layout: a field moved, dropped, or read with a new meaning.
/// Serde would take such a record on whatever fields still line up and drop
/// the rest without a word, so a hand-edited or newer-tool row would load as a
/// record nobody here understood, and every answer computed from it would be
/// quietly wrong.
///
/// The store therefore refuses it at the door, for every family it reads and
/// not only the ideation ones: requirements, rules, sources, edges, messages
/// and the rest all pass [`record_from_line`]. The refusal names the file, the
/// record if the line carries an id, and both versions, because the fix is to
/// find that line and decide what it should say.
///
/// Every write is a read first: `mutate_jsonl_locked` loads the shard, hands
/// the records to the caller, and writes the whole shard back, dropping any
/// unrecognised field on the way out. That path therefore calls this same
/// function, on the same raw JSON, before any record is built - see
/// `crate::jsonl`. A write against a shard holding an unsupported row fails
/// before the mutation runs, and the shard is left byte for byte as it was.
///
/// A line with no `schema_version` at all makes no claim about its layout;
/// there is nothing to compare, and its own deserializer says whether the
/// field was required. The version is read from the raw JSON rather than from
/// a struct because the struct is what we refuse to build until the version is
/// known.
#[rule("rule_reads_supported_version_only")]
pub fn ensure_supported_record_version(
    path: &Utf8Path,
    line_number: usize,
    value: &serde_json::Value,
) -> anyhow::Result<()> {
    let Some(version) = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    else {
        return Ok(());
    };
    if version == u64::from(SUPPORTED_SCHEMA_VERSION.0) {
        return Ok(());
    }
    let record = value
        .get("id")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| "record".to_string(), |id| format!("record {id}"));
    anyhow::bail!(
        "{path} line {line_number}: {record} has schema_version {version}, \
         but this build reads schema_version {} only",
        SUPPORTED_SCHEMA_VERSION.0
    )
}

fn read_records<T: DeserializeOwned>(
    path: &Utf8Path,
    fields: Fields,
    prepare: fn(&mut serde_json::Value),
) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(path)?;
    let mut records = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let mut value: serde_json::Value = serde_json::from_str(line)?;
        prepare(&mut value);
        records.push(record_from_line(path, index + 1, line, value, fields)?);
    }
    Ok(records)
}

const fn leave_as_written(_value: &mut serde_json::Value) {}

pub(super) fn read_jsonl<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    crate::publication::with_state_path_access(path, || read_jsonl_unlocked(path))
}

fn read_jsonl_unlocked<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    read_records(path, Fields::Open, leave_as_written)
}

pub(super) fn read_legacy_dispositions(path: &Utf8Path) -> anyhow::Result<Vec<DispositionRecord>> {
    crate::publication::with_state_path_access(path, || read_legacy_dispositions_unlocked(path))
}

fn read_legacy_dispositions_unlocked(path: &Utf8Path) -> anyhow::Result<Vec<DispositionRecord>> {
    read_records(path, Fields::Open, normalize_disposition_aliases)
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
    read_records(path, Fields::Closed, leave_as_written)
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
            let context = || {
                format!(
                    "failed to parse {shard_kind} shard {} line {}",
                    path.as_str(),
                    index + 1
                )
            };
            let value: serde_json::Value = serde_json::from_str(line).with_context(context)?;
            records.push(
                record_from_line(&path, index + 1, line, value, Fields::Open)
                    .with_context(context)?,
            );
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
            let context = || {
                format!(
                    "failed to parse edge shard {} line {}",
                    path.as_str(),
                    index + 1
                )
            };
            let value: serde_json::Value = serde_json::from_str(line).with_context(context)?;
            // Another scope's rows are dropped before they are judged: a
            // pinned graph is answerable for the scope it names and for
            // nothing else, whatever version the neighbours were written in.
            if value.get("scope_id").and_then(serde_json::Value::as_str) == Some(scope.as_str()) {
                records.push(
                    record_from_line(&path, index + 1, line, value, Fields::Closed)
                        .with_context(context)?,
                );
            }
        }
    }
    Ok(records)
}

#[cfg(test)]
mod read_guard_tests {
    use super::{record_from_line, Fields};
    use camino::Utf8Path;
    use provenance_macros::verifies;
    use serde_json::{json, Value};

    // The version range the exhaustion below runs. The guard is an equality
    // test on the version and nothing else: it is monotone in nothing, so no
    // interpolation between tried values is being claimed, and every version
    // that is not 1 takes the same branch. These four are the value below the
    // accepted one, the accepted one, the value above, and the top of the u32
    // domain, which is the whole of the behaviour there is to see.
    const VERSION_RANGE: [u32; 4] = [0, 1, 2, u32::MAX];

    // Independent restatement of the decision, listed rather than compared so
    // the oracle does not repeat the guard's shape: these are the record
    // layouts this build knows how to read. Must not be implemented by
    // calling the guard.
    const READABLE_LAYOUT_VERSIONS: [u32; 1] = [1];

    // One way of turning a stored line into a record, named for the assertion.
    type Loader = fn(&str) -> anyhow::Result<()>;

    // Every way a stored line can become a record: the read choke point's two
    // field policies, and the read a write is built on, which loads the shard
    // and then writes all of it back. None of them may load a version another
    // one refuses - least of all the write, which would rewrite the row it
    // misread.
    const LOADERS: [(&str, Loader); 3] = [
        ("an open read", load_open),
        ("a closed read", load_closed),
        ("the read inside a write", load_through_a_write),
    ];

    fn layout_is_readable_by_oracle(version: u32) -> bool {
        READABLE_LAYOUT_VERSIONS.contains(&version)
    }

    fn path() -> &'static Utf8Path {
        Utf8Path::new(".provenance/state/scopes/default/requirements/req.jsonl")
    }

    fn load(line: &str, fields: Fields) -> anyhow::Result<Value> {
        let value = serde_json::from_str(line)?;
        record_from_line(path(), 7, line, value, fields)
    }

    fn load_open(line: &str) -> anyhow::Result<()> {
        load(line, Fields::Open).map(drop)
    }

    fn load_closed(line: &str) -> anyhow::Result<()> {
        load(line, Fields::Closed).map(drop)
    }

    // The mutation empties the shard, so the loader fails loudly if the write
    // path ever stops guarding: the record reaches the caller, the file is
    // rewritten without it, and the call returns the success the assertion
    // below refuses at every version but 1.
    fn load_through_a_write(line: &str) -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let shard = camino::Utf8PathBuf::from_path_buf(dir.path().join("req.jsonl"))
            .expect("temporary directory path must be UTF-8");
        std::fs::write(&shard, format!("{line}\n"))?;
        crate::jsonl::mutate_jsonl_locked(
            &shard,
            &shard.with_extension("lock"),
            |records: &mut Vec<Value>| {
                records.clear();
                Ok(())
            },
        )
    }

    #[test]
    #[verifies("rule_reads_supported_version_only", exhaustion)]
    fn only_the_supported_version_loads() {
        for (loader_name, load_line) in LOADERS {
            for version in VERSION_RANGE {
                let line = json!({"schema_version": version, "id": "req_a"}).to_string();
                assert_eq!(
                    load_line(&line).is_ok(),
                    layout_is_readable_by_oracle(version),
                    "the guard and the decision disagree on {loader_name} at version {version}"
                );
            }
        }
    }

    #[test]
    #[verifies("rule_reads_supported_version_only", examples)]
    fn refusal_names_the_file_the_record_and_both_versions() {
        let line = json!({"schema_version": 2, "id": "req_overtime"}).to_string();
        let message = load(&line, Fields::Open).unwrap_err().to_string();

        assert_eq!(
            message,
            "\
.provenance/state/scopes/default/requirements/req.jsonl line 7: \
record req_overtime has schema_version 2, but this build reads schema_version 1 only"
        );
    }

    #[test]
    #[verifies("rule_reads_supported_version_only", examples)]
    fn refusal_says_so_even_when_the_line_carries_no_id() {
        let line = json!({"schema_version": 2}).to_string();
        let message = load(&line, Fields::Open).unwrap_err().to_string();

        assert!(message.contains("record has schema_version 2"), "{message}");
    }

    #[test]
    #[verifies("rule_reads_supported_version_only", examples)]
    fn a_line_claiming_no_version_is_left_to_its_own_deserializer() {
        let line = json!({"id": "req_a"}).to_string();

        assert!(load(&line, Fields::Open).is_ok());
    }
}
