use crate::layout::ProvenanceLayout;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io::Write;

pub struct StateSnapshot {
    _directory: tempfile::TempDir,
    layout: ProvenanceLayout,
}

impl StateSnapshot {
    pub const fn layout(&self) -> &ProvenanceLayout {
        &self.layout
    }
}

thread_local! {
    static HELD_LOCKS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
}

struct HeldPublicationLock {
    key: String,
}

impl HeldPublicationLock {
    fn new(key: String) -> Self {
        HELD_LOCKS.with(|locks| locks.borrow_mut().insert(key.clone()));
        Self { key }
    }
}

impl Drop for HeldPublicationLock {
    fn drop(&mut self) {
        HELD_LOCKS.with(|locks| locks.borrow_mut().remove(&self.key));
    }
}

pub fn with_repository_publication<R>(
    layout: &ProvenanceLayout,
    operation: impl FnOnce() -> anyhow::Result<R>,
) -> anyhow::Result<R> {
    let lock_path = layout.publication_lock_path();
    let key = lock_path.to_string();
    if HELD_LOCKS.with(|locks| locks.borrow().contains(&key)) {
        return operation();
    }
    crate::jsonl::with_advisory_lock(&lock_path, || {
        let _held_lock = HeldPublicationLock::new(key);
        recover_pending_publication(layout).and_then(|()| operation())
    })
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationPhase {
    Prepared,
    BackupCreated,
    Published,
}

#[derive(serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PublicationMarker {
    schema_version: u32,
    transaction_dir: Utf8PathBuf,
    phase: PublicationPhase,
}

pub fn write_publication_marker(
    layout: &ProvenanceLayout,
    transaction_dir: &Utf8Path,
    phase: PublicationPhase,
) -> anyhow::Result<()> {
    let transaction_dir = validated_transaction_dir(layout, transaction_dir)?;
    let marker = PublicationMarker {
        schema_version: 1,
        transaction_dir,
        phase,
    };
    let path = layout.publication_marker_path();
    std::fs::create_dir_all(layout.cache_dir())?;
    let mut temporary = tempfile::NamedTempFile::new_in(layout.cache_dir())?;
    temporary.write_all(&serde_json::to_vec(&marker)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    sync_directory(&layout.cache_dir())
}

pub fn clear_publication_marker(layout: &ProvenanceLayout) -> anyhow::Result<()> {
    let path = layout.publication_marker_path();
    if path.exists() {
        std::fs::remove_file(path)?;
        sync_directory(&layout.cache_dir())?;
    }
    Ok(())
}

pub fn recover_pending_publication(layout: &ProvenanceLayout) -> anyhow::Result<()> {
    let marker_path = layout.publication_marker_path();
    if !marker_path.exists() {
        return Ok(());
    }
    let marker: PublicationMarker = serde_json::from_str(&std::fs::read_to_string(&marker_path)?)?;
    anyhow::ensure!(
        marker.schema_version == 1,
        "unsupported publication marker version"
    );
    if matches!(marker.phase, PublicationPhase::Published) && !marker.transaction_dir.exists() {
        validate_missing_transaction_dir(layout, &marker.transaction_dir)?;
        return clear_publication_marker(layout);
    }
    let transaction_dir = validated_transaction_dir(layout, &marker.transaction_dir)?;
    let backup = transaction_dir.join("backup-state");
    if !layout.state_dir().exists() {
        anyhow::ensure!(
            backup.exists(),
            "publication recovery found neither live state nor backup state"
        );
        std::fs::rename(&backup, layout.state_dir())?;
        sync_directory(&layout.provenance_dir())?;
    }
    if transaction_dir.exists() {
        std::fs::remove_dir_all(&transaction_dir)?;
    }
    clear_publication_marker(layout)
}

fn validated_transaction_dir(
    layout: &ProvenanceLayout,
    transaction_dir: &Utf8Path,
) -> anyhow::Result<Utf8PathBuf> {
    let canonical_transactions = canonical_transactions_dir(layout)?;
    let canonical_transaction = Utf8PathBuf::from_path_buf(std::fs::canonicalize(transaction_dir)?)
        .map_err(|path| {
            anyhow::anyhow!("import transaction path is not UTF-8: {}", path.display())
        })?;
    let parent = transaction_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("publication marker transaction has no parent"))?;
    let canonical_parent = std::fs::canonicalize(parent)?;
    anyhow::ensure!(
        canonical_parent == canonical_transactions
            && canonical_transaction.parent() == Some(canonical_transactions.as_path()),
        "publication marker transaction is outside the repository cache"
    );
    Ok(canonical_transaction)
}

fn validate_missing_transaction_dir(
    layout: &ProvenanceLayout,
    transaction_dir: &Utf8Path,
) -> anyhow::Result<()> {
    let parent = transaction_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("publication marker transaction has no parent"))?;
    let canonical_transactions = canonical_transactions_dir(layout)?;
    let canonical_parent = std::fs::canonicalize(parent)?;
    anyhow::ensure!(
        canonical_parent == canonical_transactions,
        "publication marker transaction is outside the repository cache"
    );
    Ok(())
}

fn canonical_transactions_dir(layout: &ProvenanceLayout) -> anyhow::Result<Utf8PathBuf> {
    let canonical_cache = Utf8PathBuf::from_path_buf(std::fs::canonicalize(layout.cache_dir())?)
        .map_err(|path| {
            anyhow::anyhow!("repository cache path is not UTF-8: {}", path.display())
        })?;
    let canonical_transactions =
        Utf8PathBuf::from_path_buf(std::fs::canonicalize(layout.import_transactions_dir())?)
            .map_err(|path| {
                anyhow::anyhow!("import transaction path is not UTF-8: {}", path.display())
            })?;
    anyhow::ensure!(
        canonical_transactions == canonical_cache.join("import-transactions"),
        "publication marker transaction is outside the repository cache"
    );
    Ok(canonical_transactions)
}

pub fn sync_directory(path: &Utf8Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    std::fs::File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

pub fn sync_tree(path: &Utf8Path) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|path| anyhow::anyhow!("publication path is not UTF-8: {}", path.display()))?;
        if entry.file_type()?.is_dir() {
            sync_tree(&child)?;
        } else {
            std::fs::File::open(child)?.sync_all()?;
        }
    }
    sync_directory(path)
}

pub fn snapshot_state(layout: &ProvenanceLayout) -> anyhow::Result<StateSnapshot> {
    with_repository_publication(layout, || {
        let directory = tempfile::tempdir()?;
        let root = Utf8PathBuf::from_path_buf(directory.path().to_path_buf())
            .map_err(|path| anyhow::anyhow!("snapshot path is not UTF-8: {}", path.display()))?;
        let snapshot_layout = ProvenanceLayout::new(root);
        copy_tree(&layout.state_dir(), &snapshot_layout.state_dir())?;
        Ok(StateSnapshot {
            _directory: directory,
            layout: snapshot_layout,
        })
    })
}

fn copy_tree(source: &Utf8Path, destination: &Utf8Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_child = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|path| anyhow::anyhow!("state path is not UTF-8: {}", path.display()))?;
        let destination_child = destination.join(entry.file_name().to_string_lossy().as_ref());
        let file_type = std::fs::symlink_metadata(&source_child)?.file_type();
        if file_type.is_dir() {
            copy_tree(&source_child, &destination_child)?;
        } else if file_type.is_file() {
            std::fs::copy(source_child, destination_child)?;
        } else {
            anyhow::bail!("unsupported state entry: {source_child}");
        }
    }
    Ok(())
}

pub(crate) fn with_state_path_access<R>(
    path: &Utf8Path,
    operation: impl FnOnce() -> anyhow::Result<R>,
) -> anyhow::Result<R> {
    let Some(state_dir) = path.ancestors().find(|ancestor| {
        ancestor.file_name() == Some("state")
            && ancestor.parent().and_then(Utf8Path::file_name) == Some(".provenance")
    }) else {
        return operation();
    };
    let root = state_dir
        .parent()
        .and_then(Utf8Path::parent)
        .ok_or_else(|| anyhow::anyhow!("state path has no repository root"))?;
    with_repository_publication(&ProvenanceLayout::new(root), operation)
}

impl crate::state_store::StateStore {
    pub fn with_repository_publication<R>(
        &self,
        operation: impl FnOnce() -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        with_repository_publication(&self.layout, operation)
    }

    pub(crate) fn mutate_jsonl_records<T, R>(
        &self,
        path: &Utf8Path,
        mutate: impl FnOnce(&mut Vec<T>) -> anyhow::Result<R>,
    ) -> anyhow::Result<R>
    where
        T: DeserializeOwned + Serialize,
    {
        self.with_repository_publication(|| {
            let lock_path = self.layout.state_shard_lock_path(path)?;
            crate::jsonl::mutate_jsonl_locked(path, &lock_path, mutate)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_rejects_traversal_outside_import_transactions() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root.clone());
        std::fs::create_dir_all(layout.state_dir()).unwrap();
        std::fs::create_dir_all(layout.import_transactions_dir()).unwrap();
        let outside = root.join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("sentinel"), "keep").unwrap();
        std::fs::write(
            layout.publication_marker_path(),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "transaction_dir": layout.import_transactions_dir().join("../../../outside"),
                "phase": "published"
            }))
            .unwrap(),
        )
        .unwrap();

        let error = recover_pending_publication(&layout)
            .unwrap_err()
            .to_string();

        assert!(error.contains("outside the repository cache"), "{error}");
        assert!(outside.join("sentinel").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn recovery_rejects_symlinked_import_transactions_outside_repository() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().join("repo")).unwrap();
        let outside = Utf8PathBuf::from_path_buf(directory.path().join("outside")).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.state_dir()).unwrap();
        std::fs::create_dir_all(layout.cache_dir()).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, layout.import_transactions_dir()).unwrap();
        let transaction = layout.import_transactions_dir().join("Documents");
        std::fs::create_dir_all(&transaction).unwrap();
        std::fs::write(transaction.join("sentinel"), "keep").unwrap();
        std::fs::write(
            layout.publication_marker_path(),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "transaction_dir": transaction,
                "phase": "published"
            }))
            .unwrap(),
        )
        .unwrap();

        let error = recover_pending_publication(&layout)
            .unwrap_err()
            .to_string();

        assert!(error.contains("outside the repository cache"), "{error}");
        assert!(outside.join("Documents/sentinel").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn recovery_rejects_symlinked_import_transactions_inside_repository() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().join("repo")).unwrap();
        let layout = ProvenanceLayout::new(root.clone());
        let tracked = root.join("tracked");
        std::fs::create_dir_all(layout.state_dir()).unwrap();
        std::fs::create_dir_all(layout.cache_dir()).unwrap();
        std::fs::create_dir_all(tracked.join("transaction")).unwrap();
        std::fs::write(tracked.join("transaction/sentinel"), "keep").unwrap();
        std::os::unix::fs::symlink(&tracked, layout.import_transactions_dir()).unwrap();
        let transaction = layout.import_transactions_dir().join("transaction");
        std::fs::write(
            layout.publication_marker_path(),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "transaction_dir": transaction,
                "phase": "published"
            }))
            .unwrap(),
        )
        .unwrap();

        let error = recover_pending_publication(&layout)
            .unwrap_err()
            .to_string();

        assert!(error.contains("outside the repository cache"), "{error}");
        assert!(tracked.join("transaction/sentinel").is_file());
    }

    #[test]
    fn recovery_clears_published_marker_when_transaction_is_missing() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.state_dir()).unwrap();
        std::fs::create_dir_all(layout.import_transactions_dir()).unwrap();
        let transaction = layout.import_transactions_dir().join("completed");
        std::fs::write(
            layout.publication_marker_path(),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "transaction_dir": transaction,
                "phase": "published"
            }))
            .unwrap(),
        )
        .unwrap();

        recover_pending_publication(&layout).unwrap();

        assert!(layout.state_dir().is_dir());
        assert!(!layout.publication_marker_path().exists());
    }

    #[test]
    fn publication_lock_marker_is_cleared_after_panic() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.state_dir()).unwrap();
        std::fs::create_dir_all(layout.import_transactions_dir()).unwrap();

        let panic = std::panic::catch_unwind(|| {
            with_repository_publication(&layout, || -> anyhow::Result<()> {
                panic!("publication interrupted");
            })
            .unwrap();
        });
        assert!(panic.is_err());
        let key = layout.publication_lock_path().to_string();
        assert!(!HELD_LOCKS.with(|locks| locks.borrow().contains(&key)));

        let transaction = layout.import_transactions_dir().join("completed");
        std::fs::create_dir(&transaction).unwrap();
        write_publication_marker(&layout, &transaction, PublicationPhase::Published).unwrap();
        std::fs::remove_dir(transaction).unwrap();

        with_repository_publication(&layout, || Ok(())).unwrap();

        assert!(!layout.publication_marker_path().exists());
    }

    #[test]
    fn state_snapshot_waits_for_complete_publication() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.state_dir()).unwrap();
        let (midpoint_tx, midpoint_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let publisher = {
            let layout = layout.clone();
            std::thread::spawn(move || {
                with_repository_publication(&layout, || {
                    std::fs::write(layout.state_dir().join("first"), "new").unwrap();
                    midpoint_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    std::fs::write(layout.state_dir().join("second"), "new").unwrap();
                    Ok(())
                })
                .unwrap();
            })
        };
        midpoint_rx.recv().unwrap();

        let (snapshot_tx, snapshot_rx) = std::sync::mpsc::channel();
        let snapshotter =
            std::thread::spawn(move || snapshot_tx.send(snapshot_state(&layout).unwrap()).unwrap());
        assert!(snapshot_rx
            .recv_timeout(std::time::Duration::from_millis(100))
            .is_err());
        release_tx.send(()).unwrap();
        publisher.join().unwrap();
        let snapshot = snapshot_rx.recv().unwrap();
        snapshotter.join().unwrap();

        assert_eq!(
            std::fs::read_to_string(snapshot.layout().state_dir().join("first")).unwrap(),
            "new"
        );
        assert_eq!(
            std::fs::read_to_string(snapshot.layout().state_dir().join("second")).unwrap(),
            "new"
        );
    }

    #[cfg(unix)]
    #[test]
    fn state_snapshot_rejects_external_file_symlink() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().join("repo")).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.state_dir()).unwrap();
        let external = directory.path().join("secret");
        std::fs::write(&external, "do not snapshot").unwrap();
        std::os::unix::fs::symlink(&external, layout.state_dir().join("external-link")).unwrap();

        let error = match snapshot_state(&layout) {
            Ok(_) => panic!("snapshot unexpectedly followed an external symlink"),
            Err(error) => error.to_string(),
        };

        assert!(error.contains("unsupported state entry"), "{error}");
        assert_eq!(
            std::fs::read_to_string(external).unwrap(),
            "do not snapshot"
        );
    }
}
