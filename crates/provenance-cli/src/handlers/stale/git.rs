use camino::Utf8Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PathChange {
    pub old_path: String,
    pub new_path: Option<String>,
}

pub(super) fn resolve_revision(repo: &Utf8Path, revision: &str) -> anyhow::Result<String> {
    run_text(
        repo,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ],
    )
    .map(|value| value.trim().to_string())
}

pub(super) fn commit_timestamp(repo: &Utf8Path, revision: &str) -> anyhow::Result<u64> {
    run_text(repo, &["show", "-s", "--format=%ct", revision])?
        .trim()
        .parse()
        .map_err(Into::into)
}

pub(super) fn changed_paths(
    repo: &Utf8Path,
    base: &str,
    head: &str,
) -> anyhow::Result<Vec<PathChange>> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo).args([
        "diff",
        "--name-status",
        "-z",
        "--find-renames",
        base,
        head,
        "--",
    ]);
    let output = command.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git diff {base}..{head} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    parse_name_status(&output.stdout)
}

pub(super) fn file_at(
    repo: &Utf8Path,
    revision: &str,
    path: &str,
) -> anyhow::Result<Option<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["show", &format!("{revision}:{path}")])
        .output()?;
    if output.status.success() {
        return Ok(Some(String::from_utf8(output.stdout)?));
    }
    Ok(None)
}

fn run_text(repo: &Utf8Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn parse_name_status(output: &[u8]) -> anyhow::Result<Vec<PathChange>> {
    let fields: Vec<_> = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let mut index = 0;
    let mut changes = Vec::new();
    while index < fields.len() {
        let status = std::str::from_utf8(fields[index])?;
        index += 1;
        let old_path = std::str::from_utf8(
            fields
                .get(index)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("git diff omitted a path"))?,
        )?
        .to_string();
        index += 1;
        let new_path = if status.starts_with('R') || status.starts_with('C') {
            let path = std::str::from_utf8(
                fields
                    .get(index)
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("git rename omitted its destination"))?,
            )?
            .to_string();
            index += 1;
            Some(path)
        } else {
            None
        };
        changes.push(PathChange { old_path, new_path });
    }
    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::{parse_name_status, PathChange};

    #[test]
    fn parses_modified_deleted_and_renamed_paths() {
        let parsed = parse_name_status(b"M\0a.rs\0D\0b.rs\0R100\0old.rs\0new.rs\0").unwrap();
        assert_eq!(
            parsed[0],
            PathChange {
                old_path: "a.rs".into(),
                new_path: None
            }
        );
        assert_eq!(
            parsed[2],
            PathChange {
                old_path: "old.rs".into(),
                new_path: Some("new.rs".into())
            }
        );
    }
}
