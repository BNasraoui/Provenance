use camino::Utf8Path;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;

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
            .open(path)?;
        // SAFETY: flock operates on a valid file descriptor owned by `file`.
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if result == -1 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Self { file })
    }
}

impl Drop for AdvisoryLock {
    fn drop(&mut self) {
        // SAFETY: flock operates on a valid file descriptor owned by `self.file`.
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
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

fn read_jsonl_unlocked<T: DeserializeOwned>(path: &Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
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
