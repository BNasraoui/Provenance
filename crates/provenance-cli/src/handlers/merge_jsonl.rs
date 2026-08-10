use crate::output;
use camino::{Utf8Path, Utf8PathBuf};
use provenance_store::merge::{
    merge_records, read_jsonl_records, validate_merged_records, MergeOutcome,
};

/// Merges one JSONL shard and, when asked, writes the result.
///
/// `shard_path` is the repository path the merged result belongs at, which is
/// what tells the merge what type of record the file holds. Git hands a merge
/// driver three temporary files and the real path separately (`%P`), so the
/// caller must pass it; without it the merged records are written unchecked.
///
/// Conflicts are reported on stdout and then fail the command, because a git
/// merge driver that exits zero tells git the file merged cleanly. Exiting
/// non-zero leaves the path unmerged for a human to resolve.
pub(super) fn handle(
    base: &Utf8PathBuf,
    ours: &Utf8PathBuf,
    theirs: &Utf8PathBuf,
    output_path: Option<Utf8PathBuf>,
    shard_path: Option<&Utf8Path>,
    format: crate::output::OutputFormat,
) -> anyhow::Result<()> {
    let outcome = merge_records(
        &read_jsonl_records(base)?,
        &read_jsonl_records(ours)?,
        &read_jsonl_records(theirs)?,
    )?;
    let records = match &outcome {
        MergeOutcome::Clean { records } => records,
        MergeOutcome::Conflicted { partial, .. } => partial,
    };
    if let Some(shard_path) = shard_path.or(output_path.as_deref()) {
        validate_merged_records(shard_path, records)?;
    }
    if let Some(output_path) = output_path {
        provenance_store::jsonl::write_jsonl_atomic(&output_path, records)?;
    }
    output::print(format, &outcome)?;
    if let MergeOutcome::Conflicted { conflicts, .. } = &outcome {
        anyhow::bail!(
            "merge left {} conflicting record(s): {}",
            conflicts.len(),
            conflicts
                .iter()
                .map(|conflict| conflict.record_id.clone())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
    Ok(())
}
