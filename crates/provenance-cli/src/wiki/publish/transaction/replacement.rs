use super::{OutputState, StageIdentity, TransactionDirectory};
use crate::wiki::publish::{CleanupWarning, PublishError};
use camino::Utf8Path;
use std::fs::File;

pub(in crate::wiki::publish) fn replace_output(
    output: &Utf8Path,
    policy: super::super::OutputPolicy,
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
pub(in crate::wiki::publish) fn replace_output_with(
    output: &Utf8Path,
    paths: &super::TransactionPaths,
    before_rename: impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
    before_cleanup: impl FnMut(&Utf8Path) -> std::io::Result<()>,
) -> Result<Vec<CleanupWarning>, PublishError> {
    let transaction = TransactionDirectory::open(output)?;
    let output_state = super::ownership::output_identity(output)?;
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
    transaction.verify_requested_parent(output)?;
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
            return Err(super::cleanup::rollback_validation_failure(
                output,
                paths,
                &validation,
                &mut rename,
            ));
        }
        if let Err(install) = rename(&paths.stage, output) {
            return Err(super::cleanup::rollback_install_failure(
                output,
                paths,
                install,
                &mut rename,
            ));
        }
        super::cleanup::verify_installed_stage_with_backup(
            stage_identity,
            output,
            transaction,
            &mut rename,
        )?;
        let cleanup = super::cleanup::cleanup_backup(transaction, &mut before_cleanup);
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

pub(super) fn verify_stage_identity(
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

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
pub(super) fn rename_no_replace_at(parent: &File, from: &str, to: &str) -> std::io::Result<()> {
    rustix::fs::renameat_with(parent, from, parent, to, rustix::fs::RenameFlags::NOREPLACE)
        .map_err(std::io::Error::from)
}

#[cfg(target_os = "macos")]
pub(super) fn rename_no_replace_at(parent: &File, from: &str, to: &str) -> std::io::Result<()> {
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
pub(super) fn rename_no_replace_at(parent: &File, from: &str, to: &str) -> std::io::Result<()> {
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
pub(super) fn rename_no_replace_at(_parent: &File, _from: &str, _to: &str) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic no-replace rename is unavailable on this platform",
    ))
}
