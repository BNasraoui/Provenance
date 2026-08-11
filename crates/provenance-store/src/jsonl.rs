use crate::state_store::readers::ensure_supported_record_version;
use camino::Utf8Path;
use fs2::FileExt;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;

struct AdvisoryLock {
    file: File,
}

impl AdvisoryLock {
    fn acquire(path: &Utf8Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|error| anyhow::anyhow!("open advisory lock {path}: {error}"))?;
        file.lock_exclusive()
            .map_err(|error| anyhow::anyhow!("acquire advisory lock {path}: {error}"))?;
        Ok(Self { file })
    }
}

impl Drop for AdvisoryLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub fn with_advisory_lock<R>(
    path: &Utf8Path,
    operation: impl FnOnce() -> anyhow::Result<R>,
) -> anyhow::Result<R> {
    let _lock = AdvisoryLock::acquire(path)?;
    operation()
}

pub fn to_stable_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_string(value)?)
}

pub fn write_jsonl_atomic<T: Serialize>(path: &Utf8Path, records: &[T]) -> anyhow::Result<()> {
    write_jsonl_atomic_unlocked(path, records)
}

pub fn mutate_jsonl_locked<T, R>(
    path: &Utf8Path,
    lock_path: &Utf8Path,
    mutate: impl FnOnce(&mut Vec<T>) -> anyhow::Result<R>,
) -> anyhow::Result<R>
where
    T: DeserializeOwned + Serialize,
{
    let _lock = AdvisoryLock::acquire(lock_path)?;
    let mut records = read_jsonl_unlocked(path)?;
    let result = mutate(&mut records)?;
    write_jsonl_atomic_unlocked(path, &records)?;
    Ok(result)
}

/// The read a write is built on, guarded the same way an ordinary read is.
///
/// A mutation rewrites the whole shard, so every line it did not touch still
/// has to survive a round trip through a struct. A line written in a layout
/// this build does not know does not survive it: the fields the struct does
/// not recognise are dropped on the way out, which turns an unrelated `create`
/// into a silent edit of somebody else's record. So the version is read from
/// the raw JSON and judged by
/// [`ensure_supported_record_version`](crate::state_store::readers::ensure_supported_record_version),
/// the same function the read choke point calls, before any record is built.
///
/// The check runs before the caller's mutation and before anything is written,
/// so a refusal leaves the shard exactly as it was found.
fn read_jsonl_unlocked<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(path)?;
    let mut records = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let value: serde_json::Value = serde_json::from_str(line)?;
        ensure_supported_record_version(path, index + 1, &value)?;
        records.push(serde_json::from_value(value)?);
    }
    Ok(records)
}

fn write_jsonl_atomic_unlocked<T: Serialize>(path: &Utf8Path, records: &[T]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let parent = path.parent().unwrap_or_else(|| Utf8Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    for record in records {
        writeln!(temp, "{}", to_stable_json(record)?)?;
    }
    temp.persist(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    #[derive(Serialize)]
    struct Record {
        id: &'static str,
    }
    #[test]
    fn writes_newline_terminated_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let path = camino::Utf8PathBuf::from_path_buf(dir.path().join("records.jsonl")).unwrap();
        write_jsonl_atomic(&path, &[Record { id: "one" }]).unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "{\"id\":\"one\"}\n");
    }
}
