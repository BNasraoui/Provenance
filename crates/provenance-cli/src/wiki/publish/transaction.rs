use super::{manifest, CleanupWarning, PublicationOutput, PublishError};
use camino::{Utf8Path, Utf8PathBuf};
#[cfg(unix)]
use remove_dir_all::RemoveDir;
use std::fs::{File, OpenOptions};
use std::io::Write;

pub(super) enum OutputState {
    Absent,
    Existing(OutputIdentity),
}

pub(super) struct OutputIdentity(same_file::Handle);

pub(super) struct TransactionPaths {
    pub lock: Utf8PathBuf,
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
            stage: parent.join(format!(".{leaf}.provenance-wiki.stage")),
            backup: parent.join(format!(".{leaf}.provenance-wiki.backup")),
        })
    }
}

pub(super) fn acquire_lock(paths: &TransactionPaths) -> Result<File, PublishError> {
    let mut lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&paths.lock)
        .map_err(|error| PublishError::io("create publication lock", &paths.lock, error))?;
    let initialization = lock
        .write_all(b"provenance wiki publication in progress\n")
        .map_err(|error| PublishError::io("write publication lock", &paths.lock, error))
        .and_then(|()| {
            lock.sync_all()
                .map_err(|error| PublishError::io("sync publication lock", &paths.lock, error))
        });
    if let Err(primary) = initialization {
        drop(lock);
        return match std::fs::remove_file(&paths.lock) {
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
    paths: &TransactionPaths,
) -> Result<OutputState, PublishError> {
    let parent = output
        .path
        .parent()
        .filter(|path| !path.as_str().is_empty())
        .unwrap_or_else(|| Utf8Path::new("."));
    ensure_real_directory(parent, &output.path)?;

    let output_state = output_identity(&output.path)?;
    manifest::validate_output(&output.path, output.policy)?;

    if let Ok(metadata) = paths.lock.symlink_metadata() {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(PublishError::UnsafeLockPath {
                path: paths.lock.clone(),
            });
        }
        return Err(PublishError::AmbiguousArtifacts {
            paths: vec![paths.lock.clone()],
        });
    }
    let ambiguous: Vec<_> = [&paths.stage, &paths.backup]
        .into_iter()
        .filter(|path| path.symlink_metadata().is_ok())
        .cloned()
        .collect();
    if !ambiguous.is_empty() {
        return Err(PublishError::AmbiguousArtifacts { paths: ambiguous });
    }
    Ok(output_state)
}

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
    paths: &TransactionPaths,
    output_state: OutputState,
) -> Result<Vec<CleanupWarning>, PublishError> {
    replace_output_with_validation(
        output,
        paths,
        output_state,
        |_, _| Ok(()),
        |_| Ok(()),
        |backup| manifest::validate_output(backup, policy),
    )
}

#[cfg(test)]
pub(super) fn replace_output_with(
    output: &Utf8Path,
    paths: &TransactionPaths,
    before_rename: impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
    before_cleanup: impl FnMut(&Utf8Path) -> std::io::Result<()>,
) -> Result<Vec<CleanupWarning>, PublishError> {
    let output_state = output_identity(output)?;
    replace_output_with_validation(
        output,
        paths,
        output_state,
        before_rename,
        before_cleanup,
        |_| Ok(()),
    )
}

fn replace_output_with_validation(
    output: &Utf8Path,
    paths: &TransactionPaths,
    output_state: OutputState,
    mut before_rename: impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
    mut before_cleanup: impl FnMut(&Utf8Path) -> std::io::Result<()>,
    validate_backup: impl FnOnce(&Utf8Path) -> Result<(), PublishError>,
) -> Result<Vec<CleanupWarning>, PublishError> {
    let mut rename = |from: &Utf8Path, to: &Utf8Path| {
        before_rename(from, to)?;
        rename_no_replace(from, to)
    };
    if let OutputState::Existing(expected_identity) = output_state {
        rename(output, &paths.backup)
            .map_err(|error| PublishError::io("move previous output to backup", output, error))?;
        let validation =
            validate_backup(&paths.backup).and_then(|()| match output_identity(&paths.backup)? {
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
        let cleanup = cleanup_backup(&paths.backup, &mut before_cleanup);
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
    }
    Ok(Vec::new())
}

#[cfg(unix)]
fn cleanup_backup(
    backup_path: &Utf8Path,
    before_cleanup: &mut impl FnMut(&Utf8Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut backup = File::open(backup_path)?;
    let expected_identity = same_file::Handle::from_file(backup.try_clone()?)?;
    before_cleanup(backup_path)?;
    if same_file::Handle::from_path(backup_path)? != expected_identity {
        return Err(std::io::Error::other("backup path changed before cleanup"));
    }

    backup.remove_dir_contents(Some(backup_path.as_std_path()))?;
    drop(backup);
    std::fs::remove_dir(backup_path)
}

#[cfg(not(unix))]
fn cleanup_backup(
    backup_path: &Utf8Path,
    before_cleanup: &mut impl FnMut(&Utf8Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let expected_identity = same_file::Handle::from_path(backup_path)?;
    before_cleanup(backup_path)?;
    if same_file::Handle::from_path(backup_path)? != expected_identity {
        return Err(std::io::Error::other("backup path changed before cleanup"));
    }
    remove_dir_all::remove_dir_all(backup_path)
}

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
fn rename_no_replace(from: &Utf8Path, to: &Utf8Path) -> std::io::Result<()> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        from.as_std_path(),
        rustix::fs::CWD,
        to.as_std_path(),
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(std::io::Error::from)
}

#[cfg(target_os = "macos")]
fn rename_no_replace(from: &Utf8Path, to: &Utf8Path) -> std::io::Result<()> {
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

    let from = CString::new(from.as_std_path().as_os_str().as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let to = CString::new(to.as_std_path().as_os_str().as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    // Darwin's renameatx_np with RENAME_EXCL atomically refuses an existing destination.
    let result = unsafe { renameatx_np(-2, from.as_ptr(), -2, to.as_ptr(), 0x0000_0004) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn rename_no_replace(from: &Utf8Path, to: &Utf8Path) -> std::io::Result<()> {
    use std::os::raw::c_int;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> c_int;
    }

    let from: Vec<_> = from
        .as_std_path()
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let to: Vec<_> = to
        .as_std_path()
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // Omitting MOVEFILE_REPLACE_EXISTING atomically refuses an existing destination.
    let result = unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0) };
    if result != 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "macos",
    windows
)))]
fn rename_no_replace(_from: &Utf8Path, _to: &Utf8Path) -> std::io::Result<()> {
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

        let error = rename_no_replace(&stage, &output).unwrap_err();

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
