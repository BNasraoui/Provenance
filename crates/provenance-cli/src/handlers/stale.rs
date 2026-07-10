use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{RuleSeverity, ScopeId};
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) fn handle(
    repo: Utf8PathBuf,
    scope: String,
    min_age_days: u32,
    rule_severities: Option<String>,
    min_downstream_rules: u32,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let rule_severities = rule_severities
        .map(|severities| {
            severities
                .split(',')
                .map(str::trim)
                .map(RuleSeverity::parse)
                .collect()
        })
        .transpose()?
        .unwrap_or_default();
    let stale = cache::find_stale_with_options(
        &ProvenanceLayout::new(repo),
        &ScopeId::new(scope)?,
        &cache::StaleOptions {
            min_age_days,
            rule_severities,
            min_downstream_rules,
        },
    )?;
    output::print(format, &stale)?;
    Ok(())
}
