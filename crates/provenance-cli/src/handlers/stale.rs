mod evidence;
mod git;

use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{RuleSeverity, ScopeId};
use provenance_store::{cache, layout::ProvenanceLayout};

pub(super) struct Options {
    pub repo: Utf8PathBuf,
    pub scope: String,
    pub min_age_days: u32,
    pub rule_severities: Option<String>,
    pub min_downstream_rules: u32,
    pub base: Option<String>,
    pub head: String,
    pub format: OutputFormat,
}

pub(super) fn handle(options: Options) -> anyhow::Result<()> {
    let scope = ScopeId::new(options.scope)?;
    let severities = parse_severities(options.rule_severities.as_deref())?;
    let filters = cache::StaleOptions {
        min_age_days: options.min_age_days,
        rule_severities: severities,
        min_downstream_rules: options.min_downstream_rules,
    };
    let layout = ProvenanceLayout::new(options.repo);
    let resolutions = cache::find_stale_with_options(&layout, &scope, &filters)?;
    let report = evidence::analyze(
        &layout,
        &scope,
        &filters,
        options.base.as_deref(),
        &options.head,
        resolutions,
    )?;
    output::print(options.format, &report)?;
    Ok(())
}

fn parse_severities(value: Option<&str>) -> anyhow::Result<Vec<RuleSeverity>> {
    value.map_or_else(
        || Ok(Vec::new()),
        |list| {
            list.split(',')
                .map(|item| RuleSeverity::parse(item.trim()))
                .collect()
        },
    )
}
