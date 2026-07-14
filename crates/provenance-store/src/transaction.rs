use camino::{Utf8Path, Utf8PathBuf};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

static TRANSACTION_ID: AtomicU64 = AtomicU64::new(1);

/// A repository-generation write assembled in memory and published under the
/// exclusive generation lock.
pub struct StateTransaction {
    staged: BTreeMap<Utf8PathBuf, Vec<u8>>,
    journal_path: Utf8PathBuf,
    fail_commit_after: Option<usize>,
}

impl StateTransaction {
    pub(crate) const fn new(journal_path: Utf8PathBuf, fail_commit_after: Option<usize>) -> Self {
        Self {
            staged: BTreeMap::new(),
            journal_path,
            fail_commit_after,
        }
    }

    pub fn read_jsonl<T: DeserializeOwned>(&self, path: &Utf8Path) -> anyhow::Result<Vec<T>> {
        match self.staged.get(path) {
            Some(bytes) => deserialize_jsonl(bytes),
            None if path.exists() => deserialize_jsonl(&std::fs::read(path)?),
            None => Ok(Vec::new()),
        }
    }

    pub fn replace_jsonl<T: Serialize>(
        &mut self,
        path: &Utf8Path,
        records: &[T],
    ) -> anyhow::Result<()> {
        self.staged
            .insert(path.to_path_buf(), serialize_jsonl(records)?);
        Ok(())
    }

    pub fn mutate_jsonl<T, R>(
        &mut self,
        path: &Utf8Path,
        mutate: impl FnOnce(&mut Vec<T>) -> anyhow::Result<R>,
    ) -> anyhow::Result<R>
    where
        T: DeserializeOwned + Serialize,
    {
        let mut records = self.read_jsonl(path)?;
        let result = mutate(&mut records)?;
        self.replace_jsonl(path, &records)?;
        Ok(result)
    }

    pub(crate) fn commit(self) -> anyhow::Result<()> {
        let prepared = self.prepare()?;
        let journal = Journal {
            entries: prepared
                .iter()
                .map(|file| JournalEntry {
                    path: file.path.clone(),
                    backup: file.backup.clone(),
                })
                .collect(),
        };
        write_journal(&self.journal_path, &journal)?;
        activate(
            prepared,
            &self.journal_path,
            &journal,
            self.fail_commit_after,
        )
    }

    fn prepare(&self) -> anyhow::Result<Vec<PreparedFile>> {
        let id = TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
        self.staged
            .iter()
            .map(|(path, bytes)| {
                let parent = path.parent().unwrap_or_else(|| Utf8Path::new("."));
                std::fs::create_dir_all(parent)?;
                let mut temp = tempfile::NamedTempFile::new_in(parent)?;
                temp.write_all(bytes)?;
                temp.as_file().sync_all()?;
                let backup = if path.exists() {
                    let file_name = path.file_name().unwrap_or("state");
                    let backup = path.with_file_name(format!(
                        ".{file_name}.transaction-{}-{id}.backup",
                        std::process::id()
                    ));
                    copy_atomic(path, &backup)?;
                    Some(backup)
                } else {
                    None
                };
                Ok(PreparedFile {
                    path: path.clone(),
                    temp,
                    backup,
                })
            })
            .collect()
    }
}

struct PreparedFile {
    path: Utf8PathBuf,
    temp: tempfile::NamedTempFile,
    backup: Option<Utf8PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct Journal {
    entries: Vec<JournalEntry>,
}

#[derive(Serialize, Deserialize)]
struct JournalEntry {
    path: Utf8PathBuf,
    backup: Option<Utf8PathBuf>,
}

fn activate(
    prepared: Vec<PreparedFile>,
    journal_path: &Utf8Path,
    journal: &Journal,
    fail_after: Option<usize>,
) -> anyhow::Result<()> {
    for (index, prepared) in prepared.into_iter().enumerate() {
        if fail_after == Some(index) {
            rollback_and_close(journal_path, journal)?;
            anyhow::bail!("injected transaction I/O failure after {index} activations");
        }
        if let Err(error) = prepared.temp.persist(&prepared.path) {
            rollback_and_close(journal_path, journal)?;
            return Err(error.error.into());
        }
        if let Err(error) = sync_parent(&prepared.path) {
            rollback_and_close(journal_path, journal)?;
            return Err(error);
        }
    }
    if let Err(error) = std::fs::remove_file(journal_path) {
        rollback_and_close(journal_path, journal)?;
        return Err(error.into());
    }
    if let Err(error) = sync_parent(journal_path) {
        write_journal(journal_path, journal)?;
        rollback_and_close(journal_path, journal)?;
        return Err(error);
    }
    cleanup_backups(journal);
    Ok(())
}

pub(crate) fn recover(journal_path: &Utf8Path) -> anyhow::Result<()> {
    if !journal_path.exists() {
        return Ok(());
    }
    let journal: Journal = serde_json::from_slice(&std::fs::read(journal_path)?)?;
    rollback_and_close(journal_path, &journal)
}

fn rollback_and_close(journal_path: &Utf8Path, journal: &Journal) -> anyhow::Result<()> {
    rollback(journal)?;
    if journal_path.exists() {
        std::fs::remove_file(journal_path)?;
        sync_parent(journal_path)?;
    }
    cleanup_backups(journal);
    Ok(())
}

fn rollback(journal: &Journal) -> anyhow::Result<()> {
    let mut first_error = None;
    for entry in journal.entries.iter().rev() {
        let result = match &entry.backup {
            Some(backup) => copy_atomic(backup, &entry.path),
            None if entry.path.exists() => std::fs::remove_file(&entry.path).map_err(Into::into),
            None => Ok(()),
        };
        if first_error.is_none() {
            first_error = result.err();
        }
    }
    first_error.map_or_else(|| Ok(()), Err)
}

fn cleanup_backups(journal: &Journal) {
    for backup in journal
        .entries
        .iter()
        .filter_map(|entry| entry.backup.as_ref())
    {
        let _ = std::fs::remove_file(backup);
    }
}

fn copy_atomic(source: &Utf8Path, destination: &Utf8Path) -> anyhow::Result<()> {
    let parent = destination.parent().unwrap_or_else(|| Utf8Path::new("."));
    std::fs::create_dir_all(parent)?;
    let temp = tempfile::NamedTempFile::new_in(parent)?;
    std::fs::copy(source, temp.path())?;
    temp.as_file().sync_all()?;
    temp.persist(destination)?;
    sync_parent(destination)?;
    Ok(())
}

fn write_journal(path: &Utf8Path, journal: &Journal) -> anyhow::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Utf8Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut temp, journal)?;
    temp.as_file().sync_all()?;
    temp.persist(path)?;
    sync_parent(path)?;
    Ok(())
}

#[cfg(unix)]
fn sync_parent(path: &Utf8Path) -> anyhow::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Utf8Path::new("."));
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent(_path: &Utf8Path) -> anyhow::Result<()> {
    Ok(())
}

fn serialize_jsonl<T: Serialize>(records: &[T]) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for record in records {
        serde_json::to_writer(&mut bytes, record)?;
        bytes.push(b'\n');
    }
    Ok(bytes)
}

fn deserialize_jsonl<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<Vec<T>> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| Ok(serde_json::from_slice(line)?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_rolls_back_an_interrupted_generation() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let target = root.join("state.jsonl");
        let backup = root.join("state.backup");
        let journal_path = root.join("transaction.json");
        std::fs::write(&target, "new\n").unwrap();
        std::fs::write(&backup, "old\n").unwrap();
        write_journal(
            &journal_path,
            &Journal {
                entries: vec![JournalEntry {
                    path: target.clone(),
                    backup: Some(backup),
                }],
            },
        )
        .unwrap();

        recover(&journal_path).unwrap();

        assert_eq!(std::fs::read_to_string(target).unwrap(), "old\n");
        assert!(!journal_path.exists());
    }
}
