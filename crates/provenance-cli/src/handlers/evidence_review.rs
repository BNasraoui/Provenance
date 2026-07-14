mod git;
mod model;
mod sites;
mod verify;

use self::git::{Git, ResolvedRevision, RevisionReader};
use self::model::{
    Contradiction, DiffRange, EvidenceResult, EvidenceSite, Report, Summary, VerificationOutcome,
};
use crate::handlers::stale::parse_severities;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{
    cache::{DownstreamRuleQuery, RulePolicy},
    layout::ProvenanceLayout,
    state_store::StateStore,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

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

struct ReviewPolicy {
    minimum_age_days: u32,
    rules: RulePolicy,
}

struct RangeAnalysis {
    diff: DiffRange,
    evidence: Vec<EvidenceResult>,
    contradictions: Vec<Contradiction>,
    intersections: BTreeSet<String>,
}

pub(super) fn handle(options: Options) -> anyhow::Result<()> {
    let scope = ScopeId::new(options.scope)?;
    let policy = ReviewPolicy {
        minimum_age_days: options.min_age_days,
        rules: RulePolicy {
            severities: parse_severities(options.rule_severities.as_deref())?,
            minimum: options.min_downstream_rules,
        },
    };
    let layout = ProvenanceLayout::new(options.repo);
    let snapshot = StateStore::new(layout.clone()).scope_snapshot(&scope)?;
    let report = review(
        &snapshot,
        layout.repo_root(),
        options.base.as_deref(),
        &options.head,
        &policy,
        &Git,
    )?;
    output::print(options.format, &report)?;
    Ok(())
}

fn review(
    snapshot: &provenance_store::state_store::ScopeSnapshot,
    repository: &camino::Utf8Path,
    base_override: Option<&str>,
    head: &str,
    policy: &ReviewPolicy,
    reader: &dyn RevisionReader,
) -> anyhow::Result<Report> {
    let mut collected = sites::collect(snapshot, repository, base_override);
    let graph = DownstreamRuleQuery::new(&snapshot.edges, &snapshot.resolutions, &snapshot.rules);
    collected.sites.retain(|site| {
        if policy.rules.minimum == 0 && policy.rules.severities.is_empty() {
            return true;
        }
        site.ownership
            .requirement_id()
            .is_some_and(|requirement| policy.rules.matches(&graph.for_requirement(requirement)))
    });
    let graph_paths: BTreeSet<_> = collected
        .sites
        .iter()
        .map(|site| site.path.clone())
        .collect();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let mut ranges: BTreeMap<(Utf8PathBuf, ResolvedRevision), Vec<&EvidenceSite>> = BTreeMap::new();
    for site in &collected.sites {
        let base = reader.resolve(&site.repository, &site.revision)?;
        let timestamp = reader.commit_timestamp(&site.repository, &base)?.0;
        if now.saturating_sub(timestamp) / 86_400 >= u64::from(policy.minimum_age_days) {
            ranges
                .entry((site.repository.clone(), base))
                .or_default()
                .push(site);
        }
    }

    let mut diffs = Vec::new();
    let mut evidence = Vec::new();
    let mut contradictions = Vec::new();
    let mut all_intersections = BTreeSet::new();
    for ((repository, base), range_sites) in ranges {
        let analysis = analyze_range(reader, &repository, &base, head, &range_sites)?;
        diffs.push(analysis.diff);
        evidence.extend(analysis.evidence);
        contradictions.extend(analysis.contradictions);
        all_intersections.extend(analysis.intersections);
    }
    evidence.sort_by(|left, right| {
        (&left.path, &left.evidence_reference_id).cmp(&(&right.path, &right.evidence_reference_id))
    });
    contradictions
        .sort_by(|left, right| left.evidence_reference_id.cmp(&right.evidence_reference_id));
    let summary = Summary {
        graph_evidence_paths: graph_paths.len(),
        intersecting_paths: all_intersections.len(),
        evidence_reverified: evidence.len(),
        moved: evidence
            .iter()
            .filter(|item| item.status == "moved")
            .count(),
        vanished: evidence
            .iter()
            .filter(|item| item.status == "vanished")
            .count(),
        contradictions: contradictions.len(),
    };
    Ok(Report {
        diffs,
        evidence,
        contradictions,
        diagnostics: collected.diagnostics,
        summary,
    })
}

fn analyze_range(
    reader: &dyn RevisionReader,
    repository: &camino::Utf8Path,
    base: &ResolvedRevision,
    head: &str,
    sites: &[&EvidenceSite],
) -> anyhow::Result<RangeAnalysis> {
    let resolved_head = reader.resolve(repository, head)?;
    let changes = reader.changed_paths(repository, base, &resolved_head)?;
    let mut groups: BTreeMap<(&str, &str), Vec<&EvidenceSite>> = BTreeMap::new();
    let mut intersections = BTreeSet::new();
    for site in sites {
        let change = changes.iter().find(|change| {
            change.old_path == site.path
                || change
                    .new_path
                    .as_deref()
                    .is_some_and(|path| path == site.path)
        });
        if let Some(change) = change {
            let matched_old = change.old_path == site.path;
            let old = if matched_old {
                change.old_path.as_str()
            } else {
                site.path.as_str()
            };
            let current = if matched_old {
                change.new_path.as_deref().unwrap_or(&change.old_path)
            } else {
                site.path.as_str()
            };
            groups.entry((old, current)).or_default().push(site);
            intersections.insert(site.path.clone());
        }
    }
    let intersecting_paths = groups.len();
    let mut evidence = Vec::new();
    let mut contradictions = Vec::new();
    for ((old_path, current_path), group_sites) in groups {
        for verified in verify::group(
            reader,
            repository,
            base,
            &resolved_head,
            old_path,
            current_path,
            &group_sites,
        )? {
            if let Some(contradiction) = contradiction(&verified) {
                contradictions.push(contradiction);
            }
            evidence.push(project(&verified));
        }
    }
    Ok(RangeAnalysis {
        diff: DiffRange {
            base: base.0.clone(),
            head: resolved_head.0,
            changed_paths: changes.len(),
            intersecting_paths,
        },
        evidence,
        contradictions,
        intersections,
    })
}

fn project(verified: &verify::VerifiedSite<'_>) -> EvidenceResult {
    let (status, current_line, reason) = match &verified.outcome {
        VerificationOutcome::Verified { line } => (
            "verified",
            Some(*line),
            "cited line remains at its recorded location",
        ),
        VerificationOutcome::Moved { line } => (
            "moved",
            Some(*line),
            "cited line was found at a new location",
        ),
        VerificationOutcome::Vanished => {
            ("vanished", None, "exact cited line is no longer present")
        }
        VerificationOutcome::Unverifiable { reason } => ("unverifiable", None, *reason),
    };
    EvidenceResult {
        owner_type: verified.site.owner.kind.as_str(),
        owner_id: verified.site.owner.id.as_str().to_string(),
        evidence_reference_id: verified.site.reference_id.as_str().to_string(),
        source_id: verified.site.source_id.as_str().to_string(),
        repository: verified.site.repository.as_str().to_string(),
        source_revision: verified.site.source_revision.clone(),
        base_revision: verified.base.0.clone(),
        head_revision: verified.head.0.clone(),
        requirement_ownership: verified.site.ownership.kind(),
        requirement_id: verified
            .site
            .ownership
            .requirement_id()
            .map(|id| id.as_str().to_string()),
        canonical_decision_id: verified
            .site
            .ownership
            .canonical_decision_id()
            .map(|id| id.as_str().to_string()),
        path: verified.site.path.clone(),
        current_path: (verified.current_path != verified.site.path)
            .then(|| verified.current_path.to_string()),
        recorded_line: verified.site.line,
        current_line,
        status,
        reason,
    }
}

fn contradiction(verified: &verify::VerifiedSite<'_>) -> Option<Contradiction> {
    if verified.outcome != VerificationOutcome::Vanished || !verified.site.owner.ratified {
        return None;
    }
    let requirement_id = verified.site.ownership.requirement_id()?;
    Some(Contradiction {
        proposal_id: verified.site.ownership.proposal_id()?.as_str().to_string(),
        requirement_id: requirement_id.as_str().to_string(),
        requirement: verified.site.owner.title.clone()?,
        evidence_reference_id: verified.site.reference_id.as_str().to_string(),
        reason: "ratified requirement lost its cited supporting line; semantic review is required",
    })
}
