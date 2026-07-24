use super::super::{CleanupWarning, PublishError};
use super::ownership::{StageIdentity, TransactionDirectory};
use camino::Utf8Path;
use remove_dir_all::RemoveDir;
use std::fs::File;
use std::io::Write;

pub(in crate::wiki::publish) struct PublicationLock {
    file: File,
    identity: same_file::Handle,
}

impl PublicationLock {
    pub(in crate::wiki::publish) fn cleanup(
        self,
        transaction: &TransactionDirectory,
    ) -> std::io::Result<()> {
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

impl TransactionDirectory {
    pub(in crate::wiki::publish) fn remove_stage(
        &self,
        expected: &StageIdentity,
    ) -> std::io::Result<()> {
        self.rename(&self.stage_leaf, &self.stage_cleanup_leaf)?;
        if self.child_identity(&self.stage_cleanup_leaf)? != expected.0 {
            let mismatch = std::io::Error::other("staging directory path changed before cleanup");
            return match self.rename(&self.stage_cleanup_leaf, &self.stage_leaf) {
                Ok(()) => Err(mismatch),
                Err(restore) => Err(std::io::Error::other(format!(
                    "{mismatch}; restoring the replacement staging directory also failed: {restore}"
                ))),
            };
        }
        match fs_at::OpenOptions::default().rmdir_at(&self.parent, &self.stage_cleanup_leaf) {
            Ok(()) => Ok(()),
            Err(cleanup) => match self.rename(&self.stage_cleanup_leaf, &self.stage_leaf) {
                Ok(()) => Err(cleanup),
                Err(restore) => Err(std::io::Error::other(format!(
                    "{cleanup}; restoring the staging directory after cleanup failed: {restore}"
                ))),
            },
        }
    }
}

pub(in crate::wiki::publish) fn acquire_lock(
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

pub(super) fn verify_installed_stage_with_backup(
    stage_identity: &StageIdentity,
    output: &Utf8Path,
    transaction: &TransactionDirectory,
    rename: &mut impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
) -> Result<(), PublishError> {
    let paths = &transaction.paths;
    let Err(validation) = super::replacement::verify_stage_identity(
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

pub(super) fn cleanup_backup(
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

pub(super) fn cleanup_warning(path: &Utf8Path, error: &std::io::Error) -> CleanupWarning {
    CleanupWarning {
        path: path.to_path_buf(),
        action: "remove previous output backup",
        error: error.to_string(),
    }
}
