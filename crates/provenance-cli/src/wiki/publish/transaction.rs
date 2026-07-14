use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::Write as _;

pub(super) struct Publication<'a> {
    out: &'a Utf8Path,
    parent: &'a Utf8Path,
    stage: Utf8PathBuf,
    backup: Utf8PathBuf,
    journal: Utf8PathBuf,
    journal_temp: Utf8PathBuf,
    journal_contents: String,
    _lock: File,
}

impl<'a> Publication<'a> {
    pub(super) fn begin(out: &'a Utf8Path) -> anyhow::Result<Self> {
        let parent = super::output_parent(out)?;
        let name = super::output_name(out)?;
        let lock_path = parent.join(format!(".{name}.publish.lock"));
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .with_context(|| format!("failed to open wiki publication lock {lock_path}"))?;
        lock.lock_exclusive()
            .with_context(|| format!("failed to lock wiki publication {lock_path}"))?;
        let journal = parent.join(format!(".{name}.publish.json"));
        let (nonce, recovering) = transaction_nonce(&journal, name)?;
        let publication = Self {
            out,
            parent,
            stage: parent.join(format!(".{name}.stage.{nonce}")),
            backup: parent.join(format!(".{name}.backup.{nonce}")),
            journal_temp: parent.join(format!(".{name}.publish.tmp.{nonce}")),
            journal,
            journal_contents: format!(
                "provenance-wiki-publication-v1\noutput={name}\nnonce={nonce}\n"
            ),
            _lock: lock,
        };
        if recovering {
            publication.reconcile()?;
        }
        Ok(publication)
    }

    pub(super) fn create_stage(&self) -> anyhow::Result<Utf8PathBuf> {
        ensure_absent(&self.backup)?;
        let mut journal = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.journal_temp)
            .with_context(|| format!("failed to create wiki journal {}", self.journal_temp))?;
        journal.write_all(self.journal_contents.as_bytes())?;
        journal.sync_all()?;
        if let Err(error) = std::fs::hard_link(&self.journal_temp, &self.journal) {
            let _ = std::fs::remove_file(&self.journal_temp);
            return Err(error)
                .with_context(|| format!("failed to publish wiki journal {}", self.journal));
        }
        std::fs::remove_file(&self.journal_temp)
            .with_context(|| format!("failed to clean wiki journal temp {}", self.journal_temp))?;
        sync_directory(self.parent)?;
        if let Err(error) = std::fs::create_dir(&self.stage) {
            self.finish_transaction()?;
            return Err(error)
                .with_context(|| format!("failed to create wiki stage {}", self.stage));
        }
        Ok(self.stage.clone())
    }

    pub(super) fn commit(&self, stage: &Utf8Path) -> anyhow::Result<()> {
        self.commit_with(stage, |from, to| std::fs::rename(from, to))
    }

    pub(super) fn abort(&self, stage: &Utf8Path) -> anyhow::Result<()> {
        if stage.exists() && !self.backup.exists() {
            remove_artifact(stage)?;
            sync_directory(self.parent)?;
            self.finish_transaction()?;
        }
        Ok(())
    }

    fn commit_with(
        &self,
        stage: &Utf8Path,
        mut rename: impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
    ) -> anyhow::Result<()> {
        if stage != self.stage {
            bail!("wiki stage {stage} does not belong to this publication");
        }
        sync_tree(stage)?;
        if !self.out.exists() {
            rename(stage, self.out)
                .with_context(|| format!("failed to publish staged wiki {stage}"))?;
            if let Err(error) = sync_directory(self.parent) {
                self.rollback_initial(stage, &mut rename, &error)?;
                return Err(error);
            }
            self.finish_committed_transaction();
            return Ok(());
        }

        rename(self.out, &self.backup).with_context(|| {
            format!(
                "failed to move existing wiki {} to {}",
                self.out, self.backup
            )
        })?;
        if let Err(error) = sync_directory(self.parent) {
            self.restore_backup(&mut rename, &error)?;
            return Err(error);
        }
        if let Err(error) = rename(stage, self.out) {
            self.restore_backup(&mut rename, &error)?;
            return Err(error).with_context(|| format!("failed to publish staged wiki {stage}"));
        }
        if let Err(error) = sync_directory(self.parent) {
            self.rollback_replacement(stage, &mut rename, &error)?;
            return Err(error);
        }
        if let Err(error) = remove_artifact(&self.backup).and_then(|()| sync_directory(self.parent))
        {
            eprintln!(
                "warning: published wiki but failed to clean backup {}: {error:#}",
                self.backup
            );
        } else {
            self.finish_committed_transaction();
        }
        Ok(())
    }

    fn rollback_initial(
        &self,
        stage: &Utf8Path,
        rename: &mut impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
        cause: &anyhow::Error,
    ) -> anyhow::Result<()> {
        rename(self.out, stage)
            .with_context(|| format!("failed to roll back unsynced wiki after: {cause:#}"))?;
        sync_directory(self.parent)
    }

    fn restore_backup(
        &self,
        rename: &mut impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
        cause: &impl std::fmt::Display,
    ) -> anyhow::Result<()> {
        rename(&self.backup, self.out)
            .with_context(|| format!("failed to restore previous wiki after: {cause}"))?;
        sync_directory(self.parent)
    }

    fn rollback_replacement(
        &self,
        stage: &Utf8Path,
        rename: &mut impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
        cause: &anyhow::Error,
    ) -> anyhow::Result<()> {
        rename(self.out, stage).with_context(|| {
            format!("failed to move unsynced publication back after: {cause:#}")
        })?;
        self.restore_backup(rename, cause)
    }

    fn reconcile(&self) -> anyhow::Result<()> {
        match (self.out.exists(), self.backup.exists()) {
            (false, true) => {
                std::fs::rename(&self.backup, self.out).with_context(|| {
                    format!(
                        "failed to recover interrupted wiki publication {}",
                        self.out
                    )
                })?;
                sync_directory(self.parent)?;
            }
            (true, true) => {
                remove_artifact(&self.backup)?;
                sync_directory(self.parent)?;
            }
            _ => {}
        }
        if self.stage.exists() {
            remove_artifact(&self.stage)?;
            sync_directory(self.parent)?;
        }
        if self.journal_temp.exists() {
            remove_artifact(&self.journal_temp)?;
            sync_directory(self.parent)?;
        }
        self.finish_transaction()
    }

    fn finish_transaction(&self) -> anyhow::Result<()> {
        match std::fs::remove_file(&self.journal) {
            Ok(()) => sync_directory(self.parent),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error)
                .with_context(|| format!("failed to remove wiki journal {}", self.journal)),
        }
    }

    fn finish_committed_transaction(&self) {
        if let Err(error) = self.finish_transaction() {
            eprintln!(
                "warning: wiki committed but failed to clean journal {}: {error:#}",
                self.journal
            );
        }
    }
}

fn transaction_nonce(journal: &Utf8Path, output_name: &str) -> anyhow::Result<(String, bool)> {
    match std::fs::symlink_metadata(journal) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            bail!("refusing symlink wiki publication journal {journal}");
        }
        Ok(_) => {
            let contents = std::fs::read_to_string(journal)?;
            let prefix = format!("provenance-wiki-publication-v1\noutput={output_name}\nnonce=");
            let nonce = contents
                .strip_prefix(&prefix)
                .and_then(|value| value.strip_suffix('\n'))
                .filter(|value| {
                    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
                .context("unrecognized wiki publication journal")?;
            Ok((nonce.to_string(), true))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut bytes = [0_u8; 16];
            getrandom::fill(&mut bytes).map_err(|error| {
                anyhow::anyhow!("failed to generate wiki transaction nonce: {error}")
            })?;
            let nonce = bytes.iter().fold(String::new(), |mut nonce, byte| {
                use std::fmt::Write as _;
                write!(nonce, "{byte:02x}").expect("writing to a String cannot fail");
                nonce
            });
            Ok((nonce, false))
        }
        Err(error) => Err(error).context("failed to inspect wiki publication journal"),
    }
}

fn ensure_absent(path: &Utf8Path) -> anyhow::Result<()> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to inspect {path}")),
        Ok(_) => bail!("refusing colliding wiki transaction path {path}"),
    }
}

fn remove_artifact(path: &Utf8Path) -> anyhow::Result<()> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect wiki transaction artifact {path}"))?;
    if metadata.file_type().is_symlink() {
        bail!("refusing symlink wiki transaction artifact {path}");
    }
    if metadata.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
    .with_context(|| format!("failed to remove wiki transaction artifact {path}"))
}

fn sync_directory(path: &Utf8Path) -> anyhow::Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .with_context(|| format!("failed to sync wiki output parent {path}"))
}

fn sync_tree(root: &Utf8Path) -> anyhow::Result<()> {
    for entry in walkdir::WalkDir::new(root)
        .contents_first(true)
        .follow_links(false)
    {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            bail!("refusing symlink in wiki stage {}", entry.path().display());
        }
        File::open(entry.path())
            .and_then(|file| file.sync_all())
            .with_context(|| {
                format!("failed to sync wiki stage entry {}", entry.path().display())
            })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returned_second_rename_failure_restores_live_output() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let out = root.join("site");
        std::fs::create_dir(&out).unwrap();
        std::fs::write(out.join("index.html"), "old").unwrap();
        let publication = Publication::begin(&out).unwrap();
        let stage = publication.create_stage().unwrap();
        std::fs::write(stage.join("index.html"), "new").unwrap();
        let mut calls = 0;

        let error = publication
            .commit_with(&stage, |from, to| {
                calls += 1;
                if calls == 2 {
                    Err(std::io::Error::other("injected publication failure"))
                } else {
                    std::fs::rename(from, to)
                }
            })
            .unwrap_err();

        assert!(format!("{error:#}").contains("injected publication failure"));
        assert_eq!(
            std::fs::read_to_string(out.join("index.html")).unwrap(),
            "old"
        );
        assert!(!publication.backup.exists());
        publication.abort(&stage).unwrap();
        assert!(!publication.journal.exists());
    }

    #[test]
    fn startup_recovers_interruption_between_commit_renames() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let out = root.join("site");
        std::fs::create_dir(&out).unwrap();
        std::fs::write(out.join("index.html"), "old").unwrap();
        let interrupted = Publication::begin(&out).unwrap();
        let stage = interrupted.create_stage().unwrap();
        std::fs::write(stage.join("index.html"), "new").unwrap();
        std::fs::rename(&out, &interrupted.backup).unwrap();
        drop(interrupted);

        let recovered = Publication::begin(&out).unwrap();

        assert_eq!(
            std::fs::read_to_string(out.join("index.html")).unwrap(),
            "old"
        );
        assert!(!recovered.backup.exists());
        assert!(!recovered.stage.exists());
    }

    #[test]
    fn startup_finishes_interruption_after_second_commit_rename() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let out = root.join("site");
        std::fs::create_dir(&out).unwrap();
        std::fs::write(out.join("index.html"), "old").unwrap();
        let interrupted = Publication::begin(&out).unwrap();
        let stage = interrupted.create_stage().unwrap();
        std::fs::write(stage.join("index.html"), "new").unwrap();
        std::fs::rename(&out, &interrupted.backup).unwrap();
        std::fs::rename(&stage, &out).unwrap();
        drop(interrupted);

        let recovered = Publication::begin(&out).unwrap();

        assert_eq!(
            std::fs::read_to_string(out.join("index.html")).unwrap(),
            "new"
        );
        assert!(!recovered.backup.exists());
    }

    #[test]
    fn random_transaction_paths_leave_caller_fixed_siblings_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let out = root.join("site");
        let caller_stage = root.join(".site.stage");
        std::fs::create_dir(&caller_stage).unwrap();
        std::fs::write(caller_stage.join("notes.txt"), "caller owned").unwrap();

        let publication = Publication::begin(&out).unwrap();
        let generated_stage = publication.create_stage().unwrap();

        assert_eq!(
            std::fs::read_to_string(caller_stage.join("notes.txt")).unwrap(),
            "caller owned"
        );
        assert_ne!(generated_stage, caller_stage);
        publication.abort(&generated_stage).unwrap();
    }

    #[test]
    fn generated_backup_collision_is_rejected_without_claiming_caller_content() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let out = root.join("site");
        let publication = Publication::begin(&out).unwrap();
        std::fs::create_dir(&publication.backup).unwrap();
        std::fs::write(publication.backup.join("notes.txt"), "caller owned").unwrap();

        let error = publication.create_stage().unwrap_err();

        assert!(format!("{error:#}").contains("colliding wiki transaction path"));
        assert_eq!(
            std::fs::read_to_string(publication.backup.join("notes.txt")).unwrap(),
            "caller owned"
        );
        assert!(!publication.journal.exists());
    }
}
