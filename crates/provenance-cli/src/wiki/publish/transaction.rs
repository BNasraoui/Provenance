use super::{manifest, PublicationOutput, PublishError};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs::File;

mod cleanup;
mod ownership;
mod replacement;

pub(super) use ownership::{acquire_lock, preflight};
pub(super) use replacement::replace_output;
#[cfg(test)]
pub(super) use replacement::replace_output_with;

pub(super) enum OutputState {
    Absent,
    Existing(OutputIdentity),
}

pub(super) struct OutputIdentity(same_file::Handle);

pub(super) struct StageIdentity(same_file::Handle);

impl StageIdentity {
    pub(super) fn from_file(file: &File) -> std::io::Result<Self> {
        same_file::Handle::from_file(file.try_clone()?).map(Self)
    }
}

pub(super) struct PublicationLock {
    file: File,
    identity: same_file::Handle,
}

pub(super) struct TransactionDirectory {
    parent: File,
    pub paths: TransactionPaths,
    output_leaf: String,
    lock_leaf: String,
    lock_cleanup_leaf: String,
    stage_leaf: String,
    stage_cleanup_leaf: String,
    backup_leaf: String,
}

impl TransactionDirectory {
    pub(super) fn open(output: &Utf8Path) -> Result<Self, PublishError> {
        let paths = TransactionPaths::new(output)?;
        let parent_path = output
            .parent()
            .filter(|path| !path.as_str().is_empty())
            .unwrap_or_else(|| Utf8Path::new("."));
        let parent = ownership::open_or_create_parent(parent_path, output)?;
        let output_leaf = output
            .file_name()
            .expect("validated output leaf")
            .to_string();
        let leaf = |role: &str| format!(".{output_leaf}.provenance-wiki.{role}");
        Ok(Self {
            parent,
            paths,
            lock_leaf: leaf("lock"),
            lock_cleanup_leaf: leaf("lock.cleanup"),
            stage_leaf: leaf("stage"),
            stage_cleanup_leaf: leaf("stage.cleanup"),
            backup_leaf: leaf("backup"),
            output_leaf,
        })
    }

    pub(super) fn create_stage(&self) -> std::io::Result<File> {
        fs_at::OpenOptions::default().mkdir_at(&self.parent, &self.stage_leaf)
    }

    fn create_file(&self, leaf: &str) -> std::io::Result<File> {
        let mut options = fs_at::OpenOptions::default();
        options
            .write(fs_at::OpenOptionsWriteMode::Write)
            .create_new(true)
            .follow(false);
        options.open_at(&self.parent, leaf)
    }

    fn open_dir(&self, leaf: &str) -> std::io::Result<File> {
        ownership::open_child_directory_no_follow(&self.parent, leaf)
    }

    fn child_identity(&self, leaf: &str) -> std::io::Result<same_file::Handle> {
        let file = if leaf == self.lock_leaf || leaf == self.lock_cleanup_leaf {
            let mut options = fs_at::OpenOptions::default();
            options.read(true).follow(false);
            options.open_at(&self.parent, leaf)?
        } else {
            self.open_dir(leaf)?
        };
        same_file::Handle::from_file(file)
    }

    fn child_exists(&self, leaf: &str) -> std::io::Result<bool> {
        ownership::child_kind(&self.parent, leaf).map(|kind| kind.is_some())
    }

    fn rename(&self, from: &str, to: &str) -> std::io::Result<()> {
        replacement::rename_no_replace_at(&self.parent, from, to)
    }

    fn remove_file(&self, leaf: &str) -> std::io::Result<()> {
        fs_at::OpenOptions::default().unlink_at(&self.parent, leaf)
    }

    pub(super) fn validate_output(
        &self,
        leaf: &str,
        display: &Utf8Path,
        policy: super::OutputPolicy,
    ) -> Result<(), PublishError> {
        let directory = match self.open_dir(leaf) {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(PublishError::io("open wiki output", display, error)),
        };
        manifest::validate_output_handle(directory, display, policy)
    }
}

pub(super) struct TransactionPaths {
    pub lock: Utf8PathBuf,
    pub lock_cleanup: Utf8PathBuf,
    pub stage: Utf8PathBuf,
    pub stage_cleanup: Utf8PathBuf,
    pub backup: Utf8PathBuf,
}

impl TransactionPaths {
    pub(super) fn new(output: &Utf8Path) -> Result<Self, PublishError> {
        let parent = output
            .parent()
            .filter(|path| !path.as_str().is_empty())
            .unwrap_or_else(|| Utf8Path::new("."));
        let leaf = output
            .file_name()
            .ok_or_else(|| PublishError::InvalidOutputPath {
                path: output.to_path_buf(),
                detail: "path has no file name".to_string(),
            })?;
        Ok(Self {
            lock: parent.join(format!(".{leaf}.provenance-wiki.lock")),
            lock_cleanup: parent.join(format!(".{leaf}.provenance-wiki.lock.cleanup")),
            stage: parent.join(format!(".{leaf}.provenance-wiki.stage")),
            stage_cleanup: parent.join(format!(".{leaf}.provenance-wiki.stage.cleanup")),
            backup: parent.join(format!(".{leaf}.provenance-wiki.backup")),
        })
    }
}

#[cfg(test)]
mod tests;
