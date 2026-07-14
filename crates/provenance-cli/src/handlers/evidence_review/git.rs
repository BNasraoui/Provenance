use camino::Utf8Path;
use std::fmt;
use std::process::{Command, Output};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedRevision(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitTimestampSeconds(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobRead {
    Present(String),
    PathAbsent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathChange {
    pub old_path: String,
    pub new_path: Option<String>,
}

#[derive(Debug)]
pub enum GitFailure {
    Io(std::io::Error),
    Command { operation: String, stderr: String },
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidTimestamp(std::num::ParseIntError),
    InvalidDiff(String),
}

impl fmt::Display for GitFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "could not execute git: {error}"),
            Self::Command { operation, stderr } => {
                write!(formatter, "git {operation} failed: {stderr}")
            }
            Self::InvalidUtf8(error) => write!(formatter, "git returned non-UTF-8 data: {error}"),
            Self::InvalidTimestamp(error) => write!(formatter, "invalid git timestamp: {error}"),
            Self::InvalidDiff(error) => write!(formatter, "invalid git diff output: {error}"),
        }
    }
}

impl std::error::Error for GitFailure {}

pub trait RevisionReader {
    fn resolve(
        &self,
        repository: &Utf8Path,
        revision: &str,
    ) -> Result<ResolvedRevision, GitFailure>;
    fn commit_timestamp(
        &self,
        repository: &Utf8Path,
        revision: &ResolvedRevision,
    ) -> Result<CommitTimestampSeconds, GitFailure>;
    fn changed_paths(
        &self,
        repository: &Utf8Path,
        base: &ResolvedRevision,
        head: &ResolvedRevision,
    ) -> Result<Vec<PathChange>, GitFailure>;
    fn blob_at(
        &self,
        repository: &Utf8Path,
        revision: &ResolvedRevision,
        path: &str,
    ) -> Result<BlobRead, GitFailure>;
}

pub struct Git;

impl RevisionReader for Git {
    fn resolve(
        &self,
        repository: &Utf8Path,
        revision: &str,
    ) -> Result<ResolvedRevision, GitFailure> {
        let output = run(
            repository,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{revision}^{{commit}}"),
            ],
        )?;
        Ok(ResolvedRevision(text(output)?.trim().to_string()))
    }

    fn commit_timestamp(
        &self,
        repository: &Utf8Path,
        revision: &ResolvedRevision,
    ) -> Result<CommitTimestampSeconds, GitFailure> {
        let output = run(repository, &["show", "-s", "--format=%ct", &revision.0])?;
        Ok(CommitTimestampSeconds(
            text(output)?
                .trim()
                .parse()
                .map_err(GitFailure::InvalidTimestamp)?,
        ))
    }

    fn changed_paths(
        &self,
        repository: &Utf8Path,
        base: &ResolvedRevision,
        head: &ResolvedRevision,
    ) -> Result<Vec<PathChange>, GitFailure> {
        let output = run(
            repository,
            &[
                "diff",
                "--name-status",
                "-z",
                "--find-renames",
                &base.0,
                &head.0,
                "--",
            ],
        )?;
        parse_name_status(&output.stdout)
    }

    fn blob_at(
        &self,
        repository: &Utf8Path,
        revision: &ResolvedRevision,
        path: &str,
    ) -> Result<BlobRead, GitFailure> {
        let listing = run(repository, &["ls-tree", "-z", &revision.0, "--", path])?;
        if listing.stdout.is_empty() {
            return Ok(BlobRead::PathAbsent);
        }
        let output = run(repository, &["show", &format!("{}:{path}", revision.0)])?;
        Ok(BlobRead::Present(text(output)?))
    }
}

fn run(repository: &Utf8Path, args: &[&str]) -> Result<Output, GitFailure> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .map_err(GitFailure::Io)?;
    if !output.status.success() {
        return Err(GitFailure::Command {
            operation: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(output)
}

fn text(output: Output) -> Result<String, GitFailure> {
    String::from_utf8(output.stdout).map_err(GitFailure::InvalidUtf8)
}

fn parse_name_status(output: &[u8]) -> Result<Vec<PathChange>, GitFailure> {
    let fields: Vec<_> = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let mut index = 0;
    let mut changes = Vec::new();
    while index < fields.len() {
        let status = std::str::from_utf8(fields[index])
            .map_err(|error| GitFailure::InvalidDiff(error.to_string()))?;
        index += 1;
        let old_path = parse_field(&fields, index, "git diff omitted a path")?;
        index += 1;
        let new_path = if status.starts_with('R') || status.starts_with('C') {
            let path = parse_field(&fields, index, "git rename omitted its destination")?;
            index += 1;
            Some(path)
        } else {
            None
        };
        changes.push(PathChange { old_path, new_path });
    }
    Ok(changes)
}

fn parse_field(fields: &[&[u8]], index: usize, missing: &str) -> Result<String, GitFailure> {
    let field = fields
        .get(index)
        .ok_or_else(|| GitFailure::InvalidDiff(missing.to_string()))?;
    std::str::from_utf8(field)
        .map(str::to_string)
        .map_err(|error| GitFailure::InvalidDiff(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{parse_name_status, BlobRead, Git, PathChange, ResolvedRevision, RevisionReader};
    use camino::Utf8PathBuf;
    use std::process::Command;

    #[test]
    fn parses_modified_deleted_and_renamed_paths() {
        let parsed = parse_name_status(b"M\0a.rs\0D\0b.rs\0R100\0old.rs\0new.rs\0").unwrap();
        assert_eq!(
            parsed,
            vec![
                PathChange {
                    old_path: "a.rs".into(),
                    new_path: None,
                },
                PathChange {
                    old_path: "b.rs".into(),
                    new_path: None,
                },
                PathChange {
                    old_path: "old.rs".into(),
                    new_path: Some("new.rs".into()),
                },
            ]
        );
    }

    #[test]
    fn distinguishes_absent_paths_from_git_failures() {
        let directory = tempfile::tempdir().unwrap();
        let repository = Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            assert!(Command::new("git")
                .arg("-C")
                .arg(&repository)
                .args(args)
                .status()
                .unwrap()
                .success());
        }
        std::fs::write(repository.join("present.txt"), "present\n").unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["add", "."])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["commit", "-qm", "base"])
            .status()
            .unwrap()
            .success());
        let head = Git.resolve(&repository, "HEAD").unwrap();

        assert_eq!(
            Git.blob_at(&repository, &head, "absent.txt").unwrap(),
            BlobRead::PathAbsent
        );
        assert!(Git
            .blob_at(
                &repository,
                &ResolvedRevision("not-a-revision".into()),
                "absent.txt"
            )
            .is_err());
    }
}
