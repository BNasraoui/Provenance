use super::{
    OutputIdentity, OutputState, PublicationLock, PublicationOutput, TransactionDirectory,
};
use crate::wiki::publish::PublishError;
use camino::Utf8Path;
use provenance_macros::rule;
use std::fs::File;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub(in crate::wiki::publish) fn acquire_lock(
    transaction: &TransactionDirectory,
) -> Result<PublicationLock, PublishError> {
    let paths = &transaction.paths;
    let lock = transaction
        .create_file(&transaction.leaves.lock)
        .map_err(|error| PublishError::io("create publication lock", &paths.lock, error))?;
    let identity_file = lock
        .try_clone()
        .map_err(|error| PublishError::io("clone publication lock handle", &paths.lock, error))?;
    let identity = same_file::Handle::from_file(identity_file).map_err(|error| {
        PublishError::io("record publication lock identity", &paths.lock, error)
    })?;
    let mut lock = PublicationLock {
        file: lock,
        identity,
    };
    let initialization = lock
        .file
        .write_all(b"provenance wiki publication in progress\n")
        .map_err(|error| PublishError::io("write publication lock", &paths.lock, error))
        .and_then(|()| {
            lock.file
                .sync_all()
                .map_err(|error| PublishError::io("sync publication lock", &paths.lock, error))
        });
    if let Err(primary) = initialization {
        return match lock.cleanup(transaction) {
            Ok(()) => Err(primary),
            Err(cleanup) => Err(PublishError::CleanupFailed {
                primary: Box::new(primary),
                path: paths.lock.clone(),
                cleanup,
            }),
        };
    }
    Ok(lock)
}

/// Decides whether a publication may start against the path it was pointed at.
///
/// Publication replaces a whole directory tree, so it refuses to start from
/// any state it cannot reason about: a symlink at the output, which would let
/// a link redirect the replacement onto a tree the caller never named; a
/// non-directory at the output, which belongs to the caller; or any artifact
/// an interrupted earlier run left beside it (the lock, the lock cleanup, the
/// stage, the stage cleanup, or the backup).
///
/// Crash residue is reported by path and left exactly as found. Only an
/// operator can tell whether a leftover backup holds the live wiki or a
/// half-written one, so this code never adopts residue as its own work and
/// never deletes it to clear the way. The refusal is the whole response.
///
/// All five artifacts are probed before that refusal is built, so the operator
/// is told about everything an interrupted run left rather than about whichever
/// leftover happened to be looked at first. Clearing one artifact and retrying
/// only to be sent back for the next is the failure this avoids.
#[rule("rule_publish_preflight")]
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

    let mut ambiguous = Vec::new();
    let mut unsafe_lock = None;
    for (leaf, path, is_lock) in [
        (&transaction.leaves.lock, &paths.lock, true),
        (&transaction.leaves.lock_cleanup, &paths.lock_cleanup, false),
        (&transaction.leaves.stage, &paths.stage, false),
        (
            &transaction.leaves.stage_cleanup,
            &paths.stage_cleanup,
            false,
        ),
        (&transaction.leaves.backup, &paths.backup, false),
    ] {
        let Some(kind) = child_kind(&transaction.parent, leaf)
            .map_err(|error| PublishError::io("inspect publication artifact", path, error))?
        else {
            continue;
        };
        if is_lock && kind != ChildKind::File {
            unsafe_lock = Some(path.clone());
        }
        ambiguous.push(path.clone());
    }
    if !ambiguous.is_empty() {
        return Err(PublishError::AmbiguousArtifacts {
            paths: ambiguous,
            unsafe_lock,
        });
    }
    Ok(output_state)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ChildKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[cfg(unix)]
pub(super) fn child_kind(parent: &File, leaf: &str) -> std::io::Result<Option<ChildKind>> {
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
pub(super) fn child_kind(parent: &File, leaf: &str) -> std::io::Result<Option<ChildKind>> {
    #[cfg(windows)]
    use std::os::windows::fs::MetadataExt;
    let mut options = fs_at::OpenOptions::default();
    options.follow(false);
    // open_path_at grants only SYNCHRONIZE by default and ignores read();
    // the metadata call below needs FILE_READ_ATTRIBUTES or Windows refuses
    // it with ERROR_ACCESS_DENIED.
    #[cfg(windows)]
    {
        use fs_at::os::windows::OpenOptionsExt;
        const FILE_READ_ATTRIBUTES: u32 = 0x0080;
        options.desired_access(FILE_READ_ATTRIBUTES);
    }
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

impl TransactionDirectory {
    pub(super) fn verify_requested_parent(&self, output: &Utf8Path) -> Result<(), PublishError> {
        let parent_path = output
            .parent()
            .filter(|path| !path.as_str().is_empty())
            .unwrap_or_else(|| Utf8Path::new("."));
        let expected =
            same_file::Handle::from_file(self.parent.try_clone().map_err(|error| {
                PublishError::io("clone output parent handle", parent_path, error)
            })?)
            .map_err(|error| {
                PublishError::io("record output parent identity", parent_path, error)
            })?;
        let current = open_directory_no_follow(parent_path.as_std_path())
            .and_then(same_file::Handle::from_file)
            .map_err(|error| PublishError::OutputChanged {
                path: output.to_path_buf(),
                detail: format!("output parent cannot be verified: {error}"),
            })?;
        if current == expected {
            Ok(())
        } else {
            Err(PublishError::OutputChanged {
                path: output.to_path_buf(),
                detail: "output parent no longer matches the opened transaction directory"
                    .to_string(),
            })
        }
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

/// Resolve OS-level symlinks in the existing part of the parent path once,
/// before the no-follow descent. On macOS the default temp area lives under
/// `/var -> /private/var`, so refusing symlinked ancestors outright would
/// reject every standard temp path. After this resolution the descent still
/// refuses to follow symlinks, so a swap mid-walk fails closed.
fn resolve_existing_parent_prefix(parent: &Path) -> std::io::Result<PathBuf> {
    let mut existing = parent;
    let mut created_below = Vec::new();
    while !existing.try_exists()? {
        match (existing.file_name(), existing.parent()) {
            (Some(leaf), Some(next)) => {
                created_below.push(leaf.to_os_string());
                existing = next;
            }
            _ => break,
        }
    }
    let mut resolved = if existing.as_os_str().is_empty() {
        std::fs::canonicalize(".")?
    } else {
        std::fs::canonicalize(existing)?
    };
    #[cfg(windows)]
    {
        resolved = normalize_verbatim_prefix_for_walk(&resolved);
    }
    for leaf in created_below.iter().rev() {
        resolved.push(leaf);
    }
    Ok(resolved)
}

#[cfg(windows)]
fn normalize_verbatim_prefix_for_walk(path: &Path) -> PathBuf {
    use std::ffi::OsString;
    use std::path::Prefix;

    let mut components = path.components();
    let Some(Component::Prefix(prefix)) = components.next() else {
        return path.to_path_buf();
    };
    let mut normalized = match prefix.kind() {
        Prefix::VerbatimDisk(drive) => PathBuf::from(format!("{}:", char::from(drive))),
        Prefix::VerbatimUNC(server, share) => {
            let mut root = OsString::from(r"\\");
            root.push(server);
            root.push(r"\");
            root.push(share);
            PathBuf::from(root)
        }
        _ => return path.to_path_buf(),
    };
    normalized.extend(components);
    normalized
}

pub(super) fn open_or_create_parent(
    parent: &Utf8Path,
    output: &Utf8Path,
) -> Result<File, PublishError> {
    let parent_resolved = resolve_existing_parent_prefix(parent.as_std_path())
        .map_err(|error| PublishError::io("resolve output parent", parent, error))?;
    let mut components = parent_resolved.components().peekable();
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

#[cfg(unix)]
pub(super) fn open_child_directory_no_follow(parent: &File, leaf: &str) -> std::io::Result<File> {
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
pub(super) fn open_child_directory_no_follow(parent: &File, leaf: &str) -> std::io::Result<File> {
    use std::os::windows::fs::MetadataExt;

    let mut options = fs_at::OpenOptions::default();
    options.read(true).follow(false);
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
pub(super) fn open_child_directory_no_follow(parent: &File, leaf: &str) -> std::io::Result<File> {
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
