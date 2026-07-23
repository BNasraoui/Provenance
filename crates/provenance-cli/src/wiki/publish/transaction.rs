use super::{manifest, CleanupWarning, PublicationOutput, PublishError};
use camino::{Utf8Path, Utf8PathBuf};
use remove_dir_all::RemoveDir;
use std::fs::File;
use std::io::Write;

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

impl PublicationLock {
    pub(super) fn cleanup(self, transaction: &TransactionDirectory) -> std::io::Result<()> {
        transaction.rename(&transaction.lock_leaf, &transaction.lock_cleanup_leaf)?;
        if transaction.child_identity(&transaction.lock_cleanup_leaf)? != self.identity {
            let mismatch = std::io::Error::other("publication lock path changed before cleanup");
            return match transaction.rename(&transaction.lock_cleanup_leaf, &transaction.lock_leaf)
            {
                Ok(()) => Err(mismatch),
                Err(restore) => Err(std::io::Error::other(format!(
                    "{mismatch}; restoring the replacement lock path also failed: {restore}"
                ))),
            };
        }
        drop(self);
        transaction.remove_file(&transaction.lock_cleanup_leaf)
    }
}

pub(super) struct TransactionDirectory {
    parent: File,
    pub paths: TransactionPaths,
    output_leaf: String,
    lock_leaf: String,
    lock_cleanup_leaf: String,
    stage_leaf: String,
    backup_leaf: String,
}

impl TransactionDirectory {
    pub(super) fn open(output: &Utf8Path) -> Result<Self, PublishError> {
        let paths = TransactionPaths::new(output)?;
        let parent_path = output
            .parent()
            .filter(|path| !path.as_str().is_empty())
            .unwrap_or_else(|| Utf8Path::new("."));
        ensure_real_directory(parent_path, output)?;
        let parent = open_directory_no_follow(parent_path).map_err(|error| {
            PublishError::io("open output parent directory", parent_path, error)
        })?;
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
            backup_leaf: leaf("backup"),
            output_leaf,
        })
    }

    pub(super) fn create_stage(&self) -> std::io::Result<File> {
        fs_at::OpenOptions::default().mkdir_at(&self.parent, &self.stage_leaf)
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

    fn create_file(&self, leaf: &str) -> std::io::Result<File> {
        let mut options = fs_at::OpenOptions::default();
        options
            .write(fs_at::OpenOptionsWriteMode::Write)
            .create_new(true)
            .follow(false);
        options.open_at(&self.parent, leaf)
    }

    fn open_dir(&self, leaf: &str) -> std::io::Result<File> {
        open_child_directory_no_follow(&self.parent, leaf)
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
        child_kind(&self.parent, leaf).map(|kind| kind.is_some())
    }

    fn rename(&self, from: &str, to: &str) -> std::io::Result<()> {
        rename_no_replace_at(&self.parent, from, to)
    }

    fn remove_file(&self, leaf: &str) -> std::io::Result<()> {
        fs_at::OpenOptions::default().unlink_at(&self.parent, leaf)
    }

    pub(super) fn remove_stage(&self) -> std::io::Result<()> {
        fs_at::OpenOptions::default().rmdir_at(&self.parent, &self.stage_leaf)
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

#[cfg(not(unix))]
fn open_child_directory_no_follow(parent: &File, leaf: &str) -> std::io::Result<File> {
    fs_at::OpenOptions::default().open_dir_at(parent, leaf)
}

#[cfg(unix)]
fn open_directory_no_follow(path: &Utf8Path) -> std::io::Result<File> {
    rustix::fs::openat(
        rustix::fs::CWD,
        path.as_std_path(),
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
fn open_directory_no_follow(path: &Utf8Path) -> std::io::Result<File> {
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
fn open_directory_no_follow(path: &Utf8Path) -> std::io::Result<File> {
    File::open(path)
}

pub(super) struct TransactionPaths {
    pub lock: Utf8PathBuf,
    pub lock_cleanup: Utf8PathBuf,
    pub stage: Utf8PathBuf,
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
            backup: parent.join(format!(".{leaf}.provenance-wiki.backup")),
        })
    }
}

pub(super) fn acquire_lock(
    transaction: &TransactionDirectory,
) -> Result<PublicationLock, PublishError> {
    let paths = &transaction.paths;
    let lock = transaction
        .create_file(&transaction.lock_leaf)
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

pub(super) fn preflight(
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
    let file = match fs_at::OpenOptions::default().open_path_at(parent, leaf) {
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
    fn output_identity(&self, leaf: &str, display: &Utf8Path) -> Result<OutputState, PublishError> {
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
fn output_identity(output: &Utf8Path) -> Result<OutputState, PublishError> {
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

fn ensure_real_directory(parent: &Utf8Path, output: &Utf8Path) -> Result<(), PublishError> {
    let mut ancestors: Vec<_> = parent
        .ancestors()
        .filter(|path| !path.as_str().is_empty())
        .collect();
    ancestors.reverse();

    for directory in ancestors {
        let metadata = match directory.symlink_metadata() {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match std::fs::create_dir(directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => {
                        return Err(PublishError::io(
                            "create output parent directory",
                            directory,
                            error,
                        ));
                    }
                }
                directory.symlink_metadata().map_err(|error| {
                    PublishError::io("inspect created output parent", directory, error)
                })?
            }
            Err(error) => {
                return Err(PublishError::io("inspect output parent", directory, error));
            }
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(PublishError::InvalidOutputPath {
                path: output.to_path_buf(),
                detail: format!("output parent ancestor {directory} must be a real directory"),
            });
        }
    }
    Ok(())
}

pub(super) fn replace_output(
    output: &Utf8Path,
    policy: super::OutputPolicy,
    transaction: &TransactionDirectory,
    output_state: OutputState,
    stage_identity: &StageIdentity,
) -> Result<Vec<CleanupWarning>, PublishError> {
    replace_output_with_validation(
        output,
        transaction,
        output_state,
        stage_identity,
        |_, _| Ok(()),
        |_| Ok(()),
        |backup| transaction.validate_output(&transaction.backup_leaf, backup, policy),
    )
}

#[cfg(test)]
pub(super) fn replace_output_with(
    output: &Utf8Path,
    paths: &TransactionPaths,
    before_rename: impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
    before_cleanup: impl FnMut(&Utf8Path) -> std::io::Result<()>,
) -> Result<Vec<CleanupWarning>, PublishError> {
    let transaction = TransactionDirectory::open(output)?;
    let output_state = output_identity(output)?;
    let stage = File::open(&paths.stage).map_err(|error| {
        PublishError::io("open staging directory identity", &paths.stage, error)
    })?;
    let stage_identity = StageIdentity::from_file(&stage).map_err(|error| {
        PublishError::io("record staging directory identity", &paths.stage, error)
    })?;
    replace_output_with_validation(
        output,
        &transaction,
        output_state,
        &stage_identity,
        before_rename,
        before_cleanup,
        |_| Ok(()),
    )
}

fn replace_output_with_validation(
    output: &Utf8Path,
    transaction: &TransactionDirectory,
    output_state: OutputState,
    stage_identity: &StageIdentity,
    mut before_rename: impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
    mut before_cleanup: impl FnMut(&Utf8Path) -> std::io::Result<()>,
    validate_backup: impl FnOnce(&Utf8Path) -> Result<(), PublishError>,
) -> Result<Vec<CleanupWarning>, PublishError> {
    let paths = &transaction.paths;
    verify_stage_identity(
        stage_identity,
        transaction,
        &transaction.stage_leaf,
        &paths.stage,
        "staging directory was replaced during generation",
    )?;
    let mut rename = |from: &Utf8Path, to: &Utf8Path| {
        before_rename(from, to)?;
        let from_leaf = from.file_name().expect("transaction path has leaf");
        let to_leaf = to.file_name().expect("transaction path has leaf");
        transaction.rename(from_leaf, to_leaf)
    };
    if let OutputState::Existing(expected_identity) = output_state {
        rename(output, &paths.backup)
            .map_err(|error| PublishError::io("move previous output to backup", output, error))?;
        let validation = validate_backup(&paths.backup).and_then(|()| {
            match transaction.output_identity(&transaction.backup_leaf, &paths.backup)? {
                OutputState::Existing(actual_identity)
                    if actual_identity.0 == expected_identity.0 =>
                {
                    Ok(())
                }
                _ => Err(PublishError::OutputChanged {
                    path: output.to_path_buf(),
                    detail: "filesystem identity no longer matches the preflight output"
                        .to_string(),
                }),
            }
        });
        if let Err(validation) = validation {
            return match rename(&paths.backup, output) {
                Ok(()) => Err(PublishError::OutputChanged {
                    path: output.to_path_buf(),
                    detail: validation.to_string(),
                }),
                Err(rollback) => Err(PublishError::RollbackFailed {
                    output: output.to_path_buf(),
                    backup: paths.backup.clone(),
                    install: std::io::Error::other(validation.to_string()),
                    rollback,
                }),
            };
        }
        if let Err(install) = rename(&paths.stage, output) {
            return match rename(&paths.backup, output) {
                Ok(()) => Err(PublishError::ReplacementRolledBack {
                    output: output.to_path_buf(),
                    source: install,
                }),
                Err(rollback) => Err(PublishError::RollbackFailed {
                    output: output.to_path_buf(),
                    backup: paths.backup.clone(),
                    install,
                    rollback,
                }),
            };
        }
        verify_installed_stage_with_backup(stage_identity, output, transaction, &mut rename)?;
        let cleanup = cleanup_backup(transaction, &mut before_cleanup);
        if let Err(error) = cleanup {
            return Ok(vec![CleanupWarning {
                path: paths.backup.clone(),
                action: "remove previous output backup",
                error: error.to_string(),
            }]);
        }
    } else {
        rename(&paths.stage, output).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                PublishError::OutputChanged {
                    path: output.to_path_buf(),
                    detail: "output appeared after preflight".to_string(),
                }
            } else {
                PublishError::io("install completed wiki", output, error)
            }
        })?;
        if let Err(validation) = verify_stage_identity(
            stage_identity,
            transaction,
            &transaction.output_leaf,
            output,
            "installed output does not match the generated staging directory",
        ) {
            rename(output, &paths.stage).map_err(|error| {
                PublishError::io("quarantine replaced staging directory", output, error)
            })?;
            return Err(validation);
        }
    }
    Ok(Vec::new())
}

fn verify_installed_stage_with_backup(
    stage_identity: &StageIdentity,
    output: &Utf8Path,
    transaction: &TransactionDirectory,
    rename: &mut impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
) -> Result<(), PublishError> {
    let paths = &transaction.paths;
    let Err(validation) = verify_stage_identity(
        stage_identity,
        transaction,
        &transaction.output_leaf,
        output,
        "installed output does not match the generated staging directory",
    ) else {
        return Ok(());
    };
    let install = std::io::Error::other(validation.to_string());
    if let Err(quarantine) = rename(output, &paths.stage) {
        return Err(PublishError::RollbackFailed {
            output: output.to_path_buf(),
            backup: paths.backup.clone(),
            install,
            rollback: quarantine,
        });
    }
    match rename(&paths.backup, output) {
        Ok(()) => Err(validation),
        Err(rollback) => Err(PublishError::RollbackFailed {
            output: output.to_path_buf(),
            backup: paths.backup.clone(),
            install,
            rollback,
        }),
    }
}

fn verify_stage_identity(
    expected: &StageIdentity,
    transaction: &TransactionDirectory,
    leaf: &str,
    path: &Utf8Path,
    detail: &'static str,
) -> Result<(), PublishError> {
    if transaction
        .child_identity(leaf)
        .map_err(|error| PublishError::io("verify staging directory identity", path, error))?
        == expected.0
    {
        Ok(())
    } else {
        Err(PublishError::OutputChanged {
            path: path.to_path_buf(),
            detail: detail.to_string(),
        })
    }
}

fn cleanup_backup(
    transaction: &TransactionDirectory,
    before_cleanup: &mut impl FnMut(&Utf8Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let backup_path = &transaction.paths.backup;
    let mut backup = transaction.open_dir(&transaction.backup_leaf)?;
    let expected_identity = same_file::Handle::from_file(backup.try_clone()?)?;
    before_cleanup(backup_path)?;
    if transaction.child_identity(&transaction.backup_leaf)? != expected_identity {
        return Err(std::io::Error::other("backup path changed before cleanup"));
    }

    backup.remove_dir_contents(Some(backup_path.as_std_path()))?;
    drop(backup);
    fs_at::OpenOptions::default().rmdir_at(&transaction.parent, &transaction.backup_leaf)
}

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
fn rename_no_replace_at(parent: &File, from: &str, to: &str) -> std::io::Result<()> {
    rustix::fs::renameat_with(parent, from, parent, to, rustix::fs::RenameFlags::NOREPLACE)
        .map_err(std::io::Error::from)
}

#[cfg(target_os = "macos")]
fn rename_no_replace_at(parent: &File, from: &str, to: &str) -> std::io::Result<()> {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_uint};
    use std::os::unix::ffi::OsStrExt;

    extern "C" {
        fn renameatx_np(
            from_dir: c_int,
            from: *const c_char,
            to_dir: c_int,
            to: *const c_char,
            flags: c_uint,
        ) -> c_int;
    }

    use std::os::fd::AsRawFd;
    let from = CString::new(from.as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let to = CString::new(to.as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    // Darwin's renameatx_np with RENAME_EXCL atomically refuses an existing destination.
    let result = unsafe {
        renameatx_np(
            parent.as_raw_fd(),
            from.as_ptr(),
            parent.as_raw_fd(),
            to.as_ptr(),
            0x0000_0004,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn rename_no_replace_at(parent: &File, from: &str, to: &str) -> std::io::Result<()> {
    use fs_at::os::windows::OpenOptionsExt;
    use std::ffi::OsStr;
    use std::mem::size_of;
    use std::os::windows::{ffi::OsStrExt, io::AsRawHandle};
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Storage::FileSystem::{
        FileRenameInfoEx, SetFileInformationByHandle, FILE_RENAME_INFO, FILE_RENAME_INFO_0,
    };

    const DELETE_ACCESS: u32 = 0x0001_0000;
    let mut options = fs_at::OpenOptions::default();
    options.desired_access(DELETE_ACCESS).follow(false);
    let source = options.open_path_at(parent, from)?;
    let name: Vec<u16> = OsStr::new(to).encode_wide().collect();
    let name_bytes = name
        .len()
        .checked_mul(size_of::<u16>())
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let buffer_size = size_of::<FILE_RENAME_INFO>()
        .checked_add(name_bytes)
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let mut storage = vec![0_usize; buffer_size.div_ceil(size_of::<usize>())];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    unsafe {
        info.write(FILE_RENAME_INFO {
            Anonymous: FILE_RENAME_INFO_0 { Flags: 0 },
            RootDirectory: parent.as_raw_handle() as HANDLE,
            FileNameLength: u32::try_from(name_bytes)
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?,
            FileName: [0],
        });
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
            name.len(),
        );
        if SetFileInformationByHandle(
            source.as_raw_handle() as HANDLE,
            FileRenameInfoEx,
            info.cast(),
            u32::try_from(buffer_size)
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?,
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "macos",
    windows
)))]
fn rename_no_replace_at(_parent: &File, _from: &str, _to: &str) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic no-replace rename is unavailable on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_failure_leaves_every_ambiguous_tree_for_operator_recovery() {
        let temp = tempfile::tempdir().unwrap();
        let output = Utf8PathBuf::from_path_buf(temp.path().join("wiki")).unwrap();
        let paths = TransactionPaths::new(&output).unwrap();
        std::fs::create_dir(&output).unwrap();
        std::fs::write(output.join("generation"), "old").unwrap();
        std::fs::create_dir(&paths.stage).unwrap();
        std::fs::write(paths.stage.join("generation"), "new").unwrap();
        let mut rename_count = 0;

        let error = replace_output_with(
            &output,
            &paths,
            |_, _| {
                rename_count += 1;
                if rename_count > 1 {
                    Err(std::io::Error::other("injected rename failure"))
                } else {
                    Ok(())
                }
            },
            |_| Ok(()),
        )
        .unwrap_err();

        assert!(matches!(error, PublishError::RollbackFailed { .. }));
        assert!(!output.exists());
        assert_eq!(
            std::fs::read_to_string(paths.backup.join("generation")).unwrap(),
            "old"
        );
        assert_eq!(
            std::fs::read_to_string(paths.stage.join("generation")).unwrap(),
            "new"
        );
    }

    #[test]
    fn no_replace_rename_preserves_an_output_that_appeared() {
        let temp = tempfile::tempdir().unwrap();
        let output = Utf8PathBuf::from_path_buf(temp.path().join("wiki")).unwrap();
        let stage = Utf8PathBuf::from_path_buf(temp.path().join("stage")).unwrap();
        std::fs::create_dir(&output).unwrap();
        std::fs::write(output.join("caller"), "keep me").unwrap();
        std::fs::create_dir(&stage).unwrap();
        std::fs::write(stage.join("generated"), "new").unwrap();

        let parent = open_directory_no_follow(
            Utf8Path::from_path(temp.path()).expect("temporary path is UTF-8"),
        )
        .unwrap();
        let error = rename_no_replace_at(&parent, "stage", "wiki").unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(
            std::fs::read_to_string(output.join("caller")).unwrap(),
            "keep me"
        );
        assert_eq!(
            std::fs::read_to_string(stage.join("generated")).unwrap(),
            "new"
        );
    }
}
