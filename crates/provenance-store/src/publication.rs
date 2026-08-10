use crate::layout::ProvenanceLayout;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_macros::rule;
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
    prepare_publication_lock(layout)?;
    let lock_path = layout.publication_lock_path();
    let key = lock_path.to_string();
    if HELD_LOCKS.with(|locks| locks.borrow().contains(&key)) {
        return operation();
    }
    crate::jsonl::with_advisory_lock(&lock_path, || {
        let _held_lock = HeldPublicationLock::new(key);
        prepare_import_transactions_dir(layout)?;
        recover_pending_publication(layout).and_then(|()| operation())
    })
}

fn prepare_publication_lock(layout: &ProvenanceLayout) -> anyhow::Result<()> {
    let canonical_root = canonical_utf8(
        layout
            .provenance_dir()
            .parent()
            .unwrap_or_else(|| Utf8Path::new(".")),
        "repository path",
    )?;
    let provenance = layout.provenance_dir();
    create_real_directory(&provenance)?;
    let canonical_provenance = canonical_utf8(&provenance, "provenance path")?;
    anyhow::ensure!(
        canonical_provenance == canonical_root.join(".provenance"),
        "repository provenance directory is outside the repository"
    );

    let cache = layout.cache_dir();
    create_real_directory(&cache)?;
    let locks = cache.join("locks");
    create_real_directory(&locks)?;
    Ok(())
}

/// Resolves `path`, keeping the failure message that names what the path was.
fn canonical_utf8(path: &Utf8Path, description: &str) -> anyhow::Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(std::fs::canonicalize(path)?)
        .map_err(|path| anyhow::anyhow!("{description} is not UTF-8: {}", path.display()))
}

/// Makes sure `path` is a real directory, creating it if it is absent.
///
/// This is the ancestor half of `rule_recovery_stays_in_cache`: called in turn
/// on `.provenance`, the cache and the lock directory, it settles that the
/// cache itself is not reached through a symlink. Everything below the cache is
/// settled by [`recovery_dir_inside_cache`]. That the two halves meet, so that
/// a symlinked cache is refused before recovery reads a marker, is checked by
/// `a_symlinked_cache_is_refused_before_recovery_runs`.
fn create_real_directory(path: &Utf8Path) -> anyhow::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if let Err(error) = std::fs::create_dir(path) {
                anyhow::ensure!(
                    error.kind() == std::io::ErrorKind::AlreadyExists,
                    "failed to create publication lock directory {path}: {error}"
                );
            }
        }
        Err(error) => return Err(error.into()),
    }
    let metadata = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        metadata.file_type().is_dir() && !metadata.file_type().is_symlink(),
        "publication lock path contains a symlink component: {path}"
    );
    Ok(())
}

fn prepare_import_transactions_dir(layout: &ProvenanceLayout) -> anyhow::Result<()> {
    create_real_directory(&layout.import_transactions_dir())?;
    canonical_transactions_dir(layout).map(|_| ())
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

const OUTSIDE_REPOSITORY_CACHE: &str =
    "publication marker transaction is outside the repository cache";

const TRANSACTION_NOT_A_DIRECTORY: &str = "publication marker transaction is not a directory";

/// What recovery is about to do with the directory it is checking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecoveryUse {
    /// Recovery will read, rename or delete the directory, so it has to be on
    /// disk and land where the name says it does.
    Touched,
    /// The directory is already gone and recovery only clears the marker, so
    /// nothing is left to resolve beyond the place it claimed to sit.
    AlreadyGone,
}

/// Publication recovery may only touch a directory that really sits inside this
/// repository's cache, reached without crossing a symlink.
///
/// Recovery renames and deletes whole trees named by
/// `.provenance/cache/import-publication.json`, which is plain data: a stale,
/// hand-edited or hostile marker must not be able to steer that deletion at the
/// working tree or at anything outside the repository. So the directory has to
/// resolve to a direct child of the directory it claims to live in, and the
/// caller has to act on the resolved path this returns rather than the name it
/// passed in.
///
/// Resolving the name and comparing it with where it is supposed to sit settles
/// both halves of the decision at once. A `..` step lands somewhere other than
/// the container, and so does a symlink, because a symlink resolves away from
/// the name it stands under. Applied twice (the transactions directory inside
/// the cache, then the transaction inside that) this covers every component
/// below the cache. That the cache itself is a real directory is settled
/// earlier, by [`create_real_directory`].
///
/// Landing in the right place is not enough on its own: recovery renames and
/// removes whole trees, and only a directory can be one. A regular file sitting
/// under the name a marker gives resolves exactly where the name says, so the
/// containment check alone would pass it on to `remove_dir_all`, which would
/// then fail from inside the recovery it was supposed to guard. A touched entry
/// therefore has to be a directory as well as contained. Nothing is probed for
/// [`RecoveryUse::AlreadyGone`], which has no entry left to be of any kind.
#[rule("rule_recovery_stays_in_cache")]
fn recovery_dir_inside_cache(
    canonical_container: &Utf8Path,
    candidate: &Utf8Path,
    recovery_use: RecoveryUse,
) -> anyhow::Result<Utf8PathBuf> {
    let parent = candidate
        .parent()
        .ok_or_else(|| anyhow::anyhow!("publication marker transaction has no parent"))?;
    let canonical_parent = std::fs::canonicalize(parent)?;
    anyhow::ensure!(
        canonical_parent == canonical_container.as_std_path(),
        OUTSIDE_REPOSITORY_CACHE
    );
    let name = candidate
        .file_name()
        .ok_or_else(|| anyhow::anyhow!(OUTSIDE_REPOSITORY_CACHE))?;
    let contained = canonical_container.join(name);
    if recovery_use == RecoveryUse::Touched {
        let resolved = canonical_utf8(candidate, "import transaction path")?;
        anyhow::ensure!(resolved == contained, OUTSIDE_REPOSITORY_CACHE);
        anyhow::ensure!(
            std::fs::symlink_metadata(&resolved)?.is_dir(),
            TRANSACTION_NOT_A_DIRECTORY
        );
    }
    Ok(contained)
}

fn validated_transaction_dir(
    layout: &ProvenanceLayout,
    transaction_dir: &Utf8Path,
) -> anyhow::Result<Utf8PathBuf> {
    let canonical_transactions = canonical_transactions_dir(layout)?;
    recovery_dir_inside_cache(
        &canonical_transactions,
        transaction_dir,
        RecoveryUse::Touched,
    )
}

fn validate_missing_transaction_dir(
    layout: &ProvenanceLayout,
    transaction_dir: &Utf8Path,
) -> anyhow::Result<()> {
    let canonical_transactions = canonical_transactions_dir(layout)?;
    recovery_dir_inside_cache(
        &canonical_transactions,
        transaction_dir,
        RecoveryUse::AlreadyGone,
    )
    .map(|_| ())
}

fn canonical_transactions_dir(layout: &ProvenanceLayout) -> anyhow::Result<Utf8PathBuf> {
    let canonical_cache = canonical_utf8(&layout.cache_dir(), "repository cache path")?;
    recovery_dir_inside_cache(
        &canonical_cache,
        &layout.import_transactions_dir(),
        RecoveryUse::Touched,
    )
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

#[cfg(all(test, unix))]
mod containment_tests;
#[cfg(test)]
mod tests;
