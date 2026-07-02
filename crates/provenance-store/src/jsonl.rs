use camino::Utf8Path;
use serde::Serialize;
use std::io::Write;

pub fn to_stable_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_string(value)?)
}

pub fn write_jsonl_atomic<T: Serialize>(path: &Utf8Path, records: &[T]) -> anyhow::Result<()> {
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
