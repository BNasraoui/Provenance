use crate::output;
use camino::Utf8PathBuf;
use provenance_store::merge::{merge_records, read_jsonl_records, MergeOutcome};

pub(super) fn handle(
    base: &Utf8PathBuf,
    ours: &Utf8PathBuf,
    theirs: &Utf8PathBuf,
    output: Option<Utf8PathBuf>,
    format: crate::output::OutputFormat,
) -> anyhow::Result<()> {
    let outcome = merge_records(
        &read_jsonl_records(base)?,
        &read_jsonl_records(ours)?,
        &read_jsonl_records(theirs)?,
    )?;
    if let Some(output_path) = output {
        let records = match &outcome {
            MergeOutcome::Clean { records } => records,
            MergeOutcome::Conflicted { partial, .. } => partial,
        };
        provenance_store::jsonl::write_jsonl_atomic(&output_path, records)?;
    }
    output::print(format, &outcome)?;
    Ok(())
}
