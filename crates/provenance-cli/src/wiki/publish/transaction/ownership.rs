use super::super::{manifest, PublicationOutput, PublishError};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs::File;
use std::path::{Component, Path, PathBuf};

pub(in crate::wiki::publish) enum OutputState {
    Absent,
    Existing(OutputIdentity),
}

pub(in crate::wiki::publish) struct OutputIdentity(pub(super) same_file::Handle);

pub(in crate::wiki::publish) struct StageIdentity(pub(super) same_file::Handle);

impl StageIdentity {
    pub(in crate::wiki::publish) fn from_file(file: &File) -> std::io::Result<Self> {
        same_file::Handle::from_file(file.try_clone()?).map(Self)
    }
}

pub(in crate::wiki::publish) struct TransactionDirectory {
    pub(super) parent: File,
    pub(in crate::wiki::publish) paths: TransactionPaths,
    pub(super) output_leaf: String,
    pub(super) lock_leaf: String,
    pub(super) lock_cleanup_leaf: String,
    pub(super) stage_leaf: String,
    pub(super) stage_cleanup_leaf: String,
    pub(super) backup_leaf: String,
}

impl TransactionDirectory {
    pub(in crate::wiki::publish) fn open(output: &Utf8Path) -> Result<Self, PublishError> {
        let paths = TransactionPaths::new(output)?;
        let parent_path = output
            .parent()
            .filter(|path| !path.as_str().is_empty())
            .unwrap_or_else(|| Utf8Path::new("."));
        let parent = open_or_create_parent(parent_path, output)?;
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

    pub(in crate::wiki::publish) fn create_stage(&self) -> std::io::Result<File> {
        fs_at::OpenOptions::default().mkdir_at(&self.parent, &self.stage_leaf)
    }

    pub(in crate::wiki::publish) fn validate_output(
        &self,
        leaf: &str,
        display: &Utf8Path,
        policy: super::super::OutputPolicy,
    ) -> Result<(), PublishError> {
        let directory = match self.open_dir(leaf) {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(PublishError::io("open wiki output", display, error)),
        };
        manifest::validate_output_handle(directory, display, policy)
    }

    pub(super) fn create_file(&self, leaf: &str) -> std::io::Result<File> {
        let mut options = fs_at::OpenOptions::default();
        options
            .write(fs_at::OpenOptionsWriteMode::Write)
            .create_new(true)
            .follow(false);
        options.open_at(&self.parent, leaf)
    }

    pub(super) fn open_dir(&self, leaf: &str) -> std::io::Result<File> {
        open_child_directory_no_follow(&self.parent, leaf)
    }

    pub(super) fn child_identity(&self, leaf: &str) -> std::io::Result<same_file::Handle> {
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
        child_kind(&self.parent, leaf).map(|kind| kind.is_some())
    }

    pub(super) fn rename(&self, from: &str, to: &str) -> std::io::Result<()> {
        super::replacement::rename_no_replace_at(&self.parent, from, to)
    }

    pub(super) fn remove_file(&self, leaf: &str) -> std::io::Result<()> {
        fs_at::OpenOptions::default().unlink_at(&self.parent, leaf)
    }

    pub(super) fn output_identity(
        &self,
        leaf: &str,
        display: &Utf8Path,
    ) -> Result<OutputState, PublishError> {
        match self.open_dir(leaf) {
            Ok(file) => same_file::Handle::from_file(file)
                .map(OutputIdentity)
                .map(OutputState::Existing)
                .map_err(|error| PublishError::io("open wiki output identity", display, error)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(OutputState::Absent),
            Err(error) => Err(PublishError::io(
                "inspect wiki output identity",
                display,
                error,
            )),
        }
    }
}

#[cfg(unix)]
fn open_child_directory_no_follow(parent: &File, leaf: &str) -> std::io::Result<File> {
    rustix::fs::openat(
        parent,
        leaf,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map(File::from)
    .map_err(std::io::Error::from)
}

#[cfg(windows)]
fn open_child_directory_no_follow(parent: &File, leaf: &str) -> std::io::Result<File> {
    use std::os::windows::fs::MetadataExt;

    let mut options = fs_at::OpenOptions::default();
    options.follow(false);
    let directory = options.open_dir_at(parent, leaf)?;
    if directory.metadata()?.file_attributes() & 0x0000_0400 != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "directory is a reparse point",
        ));
    }
    Ok(directory)
}

#[cfg(not(any(unix, windows)))]
fn open_child_directory_no_follow(parent: &File, leaf: &str) -> std::io::Result<File> {
    let mut options = fs_at::OpenOptions::default();
    options.follow(false);
    options.open_dir_at(parent, leaf)
}

#[cfg(unix)]
pub(super) fn open_directory_no_follow(path: &Path) -> std::io::Result<File> {
    rustix::fs::openat(
        rustix::fs::CWD,
        path,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map(File::from)
    .map_err(std::io::Error::from)
}

#[cfg(windows)]
pub(super) fn open_directory_no_follow(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::MetadataExt;
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    let directory = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2 | 0x4)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    if directory.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output parent is a reparse point",
        ));
    }
    Ok(directory)
}

#[cfg(not(any(unix, windows)))]
pub(super) fn open_directory_no_follow(path: &Path) -> std::io::Result<File> {
    File::open(path)
}

pub(in crate::wiki::publish) struct TransactionPaths {
    pub(in crate::wiki::publish) lock: Utf8PathBuf,
    pub(in crate::wiki::publish) lock_cleanup: Utf8PathBuf,
    pub(in crate::wiki::publish) stage: Utf8PathBuf,
    pub(in crate::wiki::publish) stage_cleanup: Utf8PathBuf,
    pub(in crate::wiki::publish) backup: Utf8PathBuf,
}

impl TransactionPaths {
    pub(in crate::wiki::publish) fn new(output: &Utf8Path) -> Result<Self, PublishError> {
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

pub(in crate::wiki::publish) fn preflight(
    output: &PublicationOutput,
    transaction: &TransactionDirectory,
) -> Result<OutputState, PublishError> {
    let paths = &transaction.paths;
    match child_kind(&transaction.parent, &transaction.output_leaf)
        .map_err(|error| PublishError::io("inspect wiki output", &output.path, error))?
    {
        Some(ChildKind::Symlink) => {
            return Err(PublishError::OutputSymlink {
                path: output.path.clone(),
            })
        }
        Some(ChildKind::File | ChildKind::Other) => {
            return Err(PublishError::OutputNotDirectory {
                path: output.path.clone(),
            })
        }
        _ => {}
    }
    let output_state = transaction.output_identity(&transaction.output_leaf, &output.path)?;
    transaction.validate_output(&transaction.output_leaf, &output.path, output.policy)?;

    if let Some(kind) = child_kind(&transaction.parent, &transaction.lock_leaf)
        .map_err(|error| PublishError::io("inspect publication lock", &paths.lock, error))?
    {
        if kind != ChildKind::File {
            return Err(PublishError::UnsafeLockPath {
                path: paths.lock.clone(),
            });
        }
        return Err(PublishError::AmbiguousArtifacts {
            paths: vec![paths.lock.clone()],
        });
    }
    let mut ambiguous = Vec::new();
    for (leaf, path) in [
        (&transaction.lock_cleanup_leaf, &paths.lock_cleanup),
        (&transaction.stage_leaf, &paths.stage),
        (&transaction.stage_cleanup_leaf, &paths.stage_cleanup),
        (&transaction.backup_leaf, &paths.backup),
    ] {
        if transaction.child_exists(leaf).unwrap_or(true) {
            ambiguous.push(path.clone());
        }
    }
    if !ambiguous.is_empty() {
        return Err(PublishError::AmbiguousArtifacts { paths: ambiguous });
    }
    Ok(output_state)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ChildKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[cfg(unix)]
fn child_kind(parent: &File, leaf: &str) -> std::io::Result<Option<ChildKind>> {
    let stat = match rustix::fs::statat(parent, leaf, rustix::fs::AtFlags::SYMLINK_NOFOLLOW) {
        Ok(stat) => stat,
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let kind = rustix::fs::FileType::from_raw_mode(stat.st_mode);
    Ok(Some(if kind.is_dir() {
        ChildKind::Directory
    } else if kind.is_file() {
        ChildKind::File
    } else if kind.is_symlink() {
        ChildKind::Symlink
    } else {
        ChildKind::Other
    }))
}

#[cfg(not(unix))]
fn child_kind(parent: &File, leaf: &str) -> std::io::Result<Option<ChildKind>> {
    #[cfg(windows)]
    use std::os::windows::fs::MetadataExt;
    let mut options = fs_at::OpenOptions::default();
    options.follow(false);
    let file = match options.open_path_at(parent, leaf) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let metadata = file.metadata()?;
    let file_type = metadata.file_type();
    #[cfg(windows)]
    let is_symlink = metadata.file_attributes() & 0x0000_0400 != 0;
    #[cfg(not(windows))]
    let is_symlink = file_type.is_symlink();
    Ok(Some(if is_symlink {
        ChildKind::Symlink
    } else if file_type.is_dir() {
        ChildKind::Directory
    } else if file_type.is_file() {
        ChildKind::File
    } else {
        ChildKind::Other
    }))
}

#[cfg(test)]
pub(super) fn output_identity(output: &Utf8Path) -> Result<OutputState, PublishError> {
    match output.symlink_metadata() {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(OutputState::Absent)
        }
        Err(error) => {
            return Err(PublishError::io(
                "inspect wiki output identity",
                output,
                error,
            ))
        }
    }
    let identity = same_file::Handle::from_path(output.as_std_path())
        .map_err(|error| PublishError::io("open wiki output identity", output, error))?;
    Ok(OutputState::Existing(OutputIdentity(identity)))
}

fn open_or_create_parent(parent: &Utf8Path, output: &Utf8Path) -> Result<File, PublishError> {
    let mut components = parent.as_std_path().components().peekable();
    let mut current_path = PathBuf::new();
    let mut current = match components.peek().copied() {
        Some(Component::Prefix(prefix)) => {
            current_path.push(prefix.as_os_str());
            components.next();
            if matches!(components.peek(), Some(Component::RootDir)) {
                current_path.push(std::path::MAIN_SEPARATOR_STR);
                components.next();
            } else {
                return Err(PublishError::InvalidOutputPath {
                    path: output.to_path_buf(),
                    detail: "drive-relative output paths are unsupported".to_string(),
                });
            }
            open_directory_no_follow(&current_path)
                .map_err(|error| PublishError::io("open output parent root", parent, error))?
        }
        Some(Component::RootDir) => {
            current_path.push(std::path::MAIN_SEPARATOR_STR);
            components.next();
            open_directory_no_follow(&current_path)
                .map_err(|error| PublishError::io("open output parent root", parent, error))?
        }
        _ => {
            current_path.push(".");
            open_directory_no_follow(&current_path)
                .map_err(|error| PublishError::io("open current directory", parent, error))?
        }
    };

    for component in components {
        let leaf = match component {
            Component::CurDir => continue,
            Component::ParentDir => "..",
            Component::Normal(leaf) => leaf.to_str().expect("UTF-8 output parent component"),
            Component::Prefix(_) | Component::RootDir => {
                return Err(PublishError::InvalidOutputPath {
                    path: output.to_path_buf(),
                    detail: "output parent contains an invalid path component".to_string(),
                })
            }
        };
        current_path.push(leaf);
        match open_child_directory_no_follow(&current, leaf) {
            Ok(next) => current = next,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match fs_at::OpenOptions::default().mkdir_at(&current, leaf) {
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => {
                        return Err(PublishError::io(
                            "create output parent directory",
                            Utf8Path::from_path(&current_path).expect("UTF-8 output parent"),
                            error,
                        ));
                    }
                }
                current = open_child_directory_no_follow(&current, leaf).map_err(|error| {
                    PublishError::io(
                        "open created output parent directory",
                        Utf8Path::from_path(&current_path).expect("UTF-8 output parent"),
                        error,
                    )
                })?;
            }
            Err(_) => {
                return Err(PublishError::InvalidOutputPath {
                    path: output.to_path_buf(),
                    detail: format!(
                        "output parent ancestor {} must be a real directory",
                        current_path.display()
                    ),
                });
            }
        }
    }
    Ok(current)
}
