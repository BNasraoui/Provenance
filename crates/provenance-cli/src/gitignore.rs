//! Ensures a repo's `.gitignore` covers a derived, regenerable path.
//!
//! Used for output directories that should never be committed by accident
//! (the generated wiki, for example) without requiring the caller to manage
//! `.gitignore` by hand.

use camino::Utf8Path;
use std::io::Write;

/// Appends `pattern` to `<repo>/.gitignore`, creating the file if it does
/// not exist. Idempotent: does nothing if a line matching `pattern` exactly
/// (ignoring surrounding whitespace) is already present. Returns whether the
/// file was created or modified.
pub fn ensure_ignored(repo: &Utf8Path, pattern: &str) -> anyhow::Result<bool> {
    let path = repo.join(".gitignore");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == pattern) {
        return Ok(false);
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    if !existing.is_empty() && !existing.ends_with('\n') {
        writeln!(file)?;
    }
    writeln!(file, "{pattern}")?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    fn repo_dir() -> (tempfile::TempDir, Utf8PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let repo = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        (dir, repo)
    }

    #[test]
    fn creates_gitignore_when_missing_and_adds_the_pattern() {
        let (_dir, repo) = repo_dir();
        let changed = ensure_ignored(&repo, ".provenance/wiki/").unwrap();
        assert!(changed);
        let contents = std::fs::read_to_string(repo.join(".gitignore")).unwrap();
        assert_eq!(contents, ".provenance/wiki/\n");
    }

    #[test]
    fn appends_to_an_existing_gitignore_without_disturbing_its_content() {
        let (_dir, repo) = repo_dir();
        std::fs::write(repo.join(".gitignore"), "target/\n*.db\n").unwrap();
        let changed = ensure_ignored(&repo, ".provenance/wiki/").unwrap();
        assert!(changed);
        let contents = std::fs::read_to_string(repo.join(".gitignore")).unwrap();
        assert_eq!(contents, "target/\n*.db\n.provenance/wiki/\n");
    }

    #[test]
    fn adds_a_newline_before_appending_when_the_file_lacks_a_trailing_one() {
        let (_dir, repo) = repo_dir();
        std::fs::write(repo.join(".gitignore"), "target/").unwrap();
        ensure_ignored(&repo, ".provenance/wiki/").unwrap();
        let contents = std::fs::read_to_string(repo.join(".gitignore")).unwrap();
        assert_eq!(contents, "target/\n.provenance/wiki/\n");
    }

    #[test]
    fn is_idempotent_when_the_pattern_is_already_present() {
        let (_dir, repo) = repo_dir();
        std::fs::write(repo.join(".gitignore"), ".provenance/wiki/\n").unwrap();
        let changed = ensure_ignored(&repo, ".provenance/wiki/").unwrap();
        assert!(!changed);
        let contents = std::fs::read_to_string(repo.join(".gitignore")).unwrap();
        assert_eq!(contents, ".provenance/wiki/\n");
    }

    #[test]
    fn matches_the_pattern_regardless_of_surrounding_whitespace() {
        let (_dir, repo) = repo_dir();
        std::fs::write(repo.join(".gitignore"), "  .provenance/wiki/  \n").unwrap();
        let changed = ensure_ignored(&repo, ".provenance/wiki/").unwrap();
        assert!(!changed);
    }
}
