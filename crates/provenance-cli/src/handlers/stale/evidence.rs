use super::git;
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use provenance_core::{
    CanonicalArtifactType, Edge, EdgeType, IdeationEvidenceReference, IdeationTargetType, NodeType,
    PromotionDecision, PromotionState, ProposalType, Rule, ScopeId,
};
use provenance_store::{cache::StaleOptions, layout::ProvenanceLayout, state_store::StateStore};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, serde::Serialize)]
pub(super) struct Report {
    diffs: Vec<DiffRange>,
    evidence: Vec<EvidenceResult>,
    contradictions: Vec<Contradiction>,
    stale_resolutions: Vec<provenance_store::cache::StaleItem>,
    diagnostics: Vec<String>,
    summary: Summary,
}

#[derive(Debug, serde::Serialize)]
struct DiffRange {
    base: String,
    head: String,
    changed_paths: usize,
    intersecting_paths: usize,
}

#[derive(Debug, serde::Serialize)]
struct EvidenceResult {
    owner_type: &'static str,
    owner_id: String,
    evidence_reference_id: String,
    path: String,
    current_path: Option<String>,
    recorded_line: Option<u32>,
    current_line: Option<u32>,
    status: &'static str,
    reason: String,
}

#[derive(Debug, serde::Serialize)]
struct Contradiction {
    proposal_id: String,
    requirement_id: String,
    requirement: String,
    evidence_reference_id: String,
    reason: String,
}

#[derive(Debug, serde::Serialize)]
struct Summary {
    graph_evidence_paths: usize,
    intersecting_paths: usize,
    evidence_reverified: usize,
    moved: usize,
    vanished: usize,
    contradictions: usize,
}

#[derive(Clone)]
struct EvidenceOwner {
    owner_type: &'static str,
    owner_id: String,
    source_id: String,
    requirement_id: Option<String>,
    requirement: Option<String>,
    ratified: bool,
    references: Vec<IdeationEvidenceReference>,
}

struct RangeAnalysis {
    diff: DiffRange,
    evidence: Vec<EvidenceResult>,
    contradictions: Vec<Contradiction>,
    intersecting_paths: BTreeSet<String>,
}

struct GroupedOwners {
    by_base: BTreeMap<String, Vec<EvidenceOwner>>,
    diagnostics: Vec<String>,
}

pub(super) fn analyze(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
    filters: &StaleOptions,
    base_override: Option<&str>,
    head: &str,
    stale_resolutions: Vec<provenance_store::cache::StaleItem>,
) -> anyhow::Result<Report> {
    let store = StateStore::new(layout.clone());
    let sources: BTreeMap<_, _> = store
        .list_sources(scope)?
        .into_iter()
        .map(|source| (source.id.as_str().to_string(), source))
        .collect();
    let edges: Vec<_> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| edge.scope_id == *scope)
        .collect();
    let rules = store.list_rules(scope)?;
    let mut owners = owners(&store, scope)?;
    owners.retain(|owner| graph_filters_match(owner, &edges, &rules, filters));
    let graph_paths = evidence_paths(&owners).into_iter().collect::<BTreeSet<_>>();

    let grouped = group_by_base(layout, owners, &sources, base_override, filters)?;

    let resolved_head = if grouped.by_base.is_empty() {
        head.to_string()
    } else {
        git::resolve_revision(layout.repo_root(), head)?
    };

    let mut diffs = Vec::new();
    let mut results = Vec::new();
    let mut contradictions = Vec::new();
    let mut intersecting = BTreeSet::new();
    for (base, owners) in grouped.by_base {
        let analysis = analyze_range(layout.repo_root(), &base, &resolved_head, &owners)?;
        diffs.push(analysis.diff);
        results.extend(analysis.evidence);
        contradictions.extend(analysis.contradictions);
        intersecting.extend(analysis.intersecting_paths);
    }
    results.sort_by(|a, b| {
        (&a.path, &a.evidence_reference_id).cmp(&(&b.path, &b.evidence_reference_id))
    });
    contradictions.sort_by(|a, b| a.evidence_reference_id.cmp(&b.evidence_reference_id));
    let summary = Summary {
        graph_evidence_paths: graph_paths.len(),
        intersecting_paths: intersecting.len(),
        evidence_reverified: results.len(),
        moved: results.iter().filter(|item| item.status == "moved").count(),
        vanished: results
            .iter()
            .filter(|item| item.status == "vanished")
            .count(),
        contradictions: contradictions.len(),
    };
    Ok(Report {
        diffs,
        evidence: results,
        contradictions,
        stale_resolutions,
        diagnostics: grouped.diagnostics,
        summary,
    })
}

fn group_by_base(
    layout: &ProvenanceLayout,
    owners: Vec<EvidenceOwner>,
    sources: &BTreeMap<String, provenance_core::Source>,
    base_override: Option<&str>,
    filters: &StaleOptions,
) -> anyhow::Result<GroupedOwners> {
    let mut by_base: BTreeMap<String, Vec<EvidenceOwner>> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for owner in owners {
        let base = base_override.map(ToOwned::to_owned).or_else(|| {
            sources
                .get(&owner.source_id)
                .and_then(|source| source.commit_pin.clone())
        });
        let Some(base) = base else {
            diagnostics.push(format!(
                "{} {} was not diffed because source {} has no commit pin",
                owner.owner_type, owner.owner_id, owner.source_id
            ));
            continue;
        };
        let resolved_base = git::resolve_revision(layout.repo_root(), &base)?;
        if source_old_enough(layout.repo_root(), &resolved_base, filters.min_age_days)? {
            by_base.entry(resolved_base).or_default().push(owner);
        }
    }
    Ok(GroupedOwners {
        by_base,
        diagnostics,
    })
}

fn analyze_range(
    repo: &Utf8Path,
    base: &str,
    head: &str,
    owners: &[EvidenceOwner],
) -> anyhow::Result<RangeAnalysis> {
    let paths = evidence_paths(owners);
    let evidence_paths: BTreeSet<_> = paths.iter().map(String::as_str).collect();
    let all_changes = git::changed_paths(repo, base, head)?;
    let path_changes: Vec<_> = all_changes
        .iter()
        .filter(|change| {
            evidence_paths.contains(change.old_path.as_str())
                || change
                    .new_path
                    .as_deref()
                    .is_some_and(|path| evidence_paths.contains(path))
        })
        .collect();
    let change_by_path: BTreeMap<_, _> = path_changes
        .iter()
        .map(|change| (change.old_path.as_str(), change.new_path.as_deref()))
        .collect();
    let mut evidence = Vec::new();
    let mut contradictions = Vec::new();
    let mut intersecting_paths = BTreeSet::new();
    for owner in owners {
        for reference in &owner.references {
            let Some(path) = reference.file_path.as_deref().and_then(normalize_path) else {
                continue;
            };
            let Some(new_path) = change_by_path.get(path.as_str()) else {
                continue;
            };
            intersecting_paths.insert(path.clone());
            let result = reverify(repo, base, head, owner, reference, &path, *new_path)?;
            if let Some(contradiction) = contradiction(owner, reference, &result) {
                contradictions.push(contradiction);
            }
            evidence.push(result);
        }
    }
    Ok(RangeAnalysis {
        diff: DiffRange {
            base: base.to_string(),
            head: head.to_string(),
            changed_paths: all_changes.len(),
            intersecting_paths: intersecting_paths.len(),
        },
        evidence,
        contradictions,
        intersecting_paths,
    })
}

fn contradiction(
    owner: &EvidenceOwner,
    reference: &IdeationEvidenceReference,
    result: &EvidenceResult,
) -> Option<Contradiction> {
    if result.status != "vanished" || !owner.ratified {
        return None;
    }
    Some(Contradiction {
        proposal_id: owner.owner_id.clone(),
        requirement_id: owner.requirement_id.clone()?,
        requirement: owner.requirement.clone()?,
        evidence_reference_id: reference.reference_id.as_str().to_string(),
        reason: "ratified requirement lost its cited supporting line; semantic review is required"
            .into(),
    })
}

fn owners(store: &StateStore, scope: &ScopeId) -> anyhow::Result<Vec<EvidenceOwner>> {
    let mut owners = Vec::new();
    let canonical_requirements: BTreeMap<_, _> = store
        .list_promotion_decisions(scope)?
        .into_iter()
        .filter(|decision| decision.decision == PromotionDecision::Accepted)
        .filter_map(|decision| {
            let artifact = decision.canonical_artifact?;
            (artifact.artifact_type == CanonicalArtifactType::Requirement).then(|| {
                (
                    decision.proposal_id.as_str().to_string(),
                    artifact.artifact_id.as_str().to_string(),
                )
            })
        })
        .collect();
    for proposal in store.list_proposal_cards(scope)? {
        let source_id = proposal
            .traceability
            .source_ids
            .first()
            .map(|id| id.as_str().to_string());
        let requirement_id = canonical_requirements
            .get(proposal.id.as_str())
            .cloned()
            .or_else(|| {
                (proposal.traceability.target.artifact_type == IdeationTargetType::Requirement)
                    .then(|| {
                        proposal
                            .traceability
                            .target
                            .artifact_id
                            .as_str()
                            .to_string()
                    })
            })
            .or_else(|| {
                (proposal.promotion_state == PromotionState::Accepted
                    && proposal.proposal_type == ProposalType::RequirementCandidate)
                    .then(|| proposal.id.as_str().to_string())
            });
        if let Some(source_id) = source_id {
            owners.push(EvidenceOwner {
                owner_type: "proposal",
                owner_id: proposal.id.as_str().to_string(),
                source_id,
                requirement_id,
                requirement: Some(proposal.title),
                ratified: proposal.promotion_state == PromotionState::Accepted,
                references: proposal.traceability.evidence_references,
            });
        }
    }
    for contribution in store.list_contributions(scope)? {
        if contribution.target.artifact_type == IdeationTargetType::Source {
            owners.push(EvidenceOwner {
                owner_type: "contribution",
                owner_id: contribution.id.as_str().to_string(),
                source_id: contribution.target.artifact_id.as_str().to_string(),
                requirement_id: None,
                requirement: None,
                ratified: false,
                references: contribution.evidence_references,
            });
        }
    }
    Ok(owners)
}

fn graph_filters_match(
    owner: &EvidenceOwner,
    edges: &[Edge],
    rules: &[Rule],
    options: &StaleOptions,
) -> bool {
    if options.rule_severities.is_empty() && options.min_downstream_rules == 0 {
        return true;
    }
    let Some(requirement_id) = owner.requirement_id.as_deref() else {
        return false;
    };
    let count = rules
        .iter()
        .filter(|rule| {
            options.rule_severities.is_empty() || options.rule_severities.contains(&rule.severity)
        })
        .filter(|rule| {
            edges.iter().any(|edge| {
                edge.edge_type == EdgeType::Produces
                    && edge.from_type == NodeType::Requirement
                    && edge.from_id.as_str() == requirement_id
                    && edge.to_type == NodeType::Rule
                    && edge.to_id == rule.id
            })
        })
        .count();
    count >= options.min_downstream_rules as usize
        && (options.rule_severities.is_empty() || count > 0)
}

fn source_old_enough(repo: &Utf8Path, base: &str, min_age_days: u32) -> anyhow::Result<bool> {
    if min_age_days == 0 {
        return Ok(true);
    }
    let timestamp = git::commit_timestamp(repo, base)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(now.saturating_sub(timestamp) / 86_400 >= u64::from(min_age_days))
}

fn evidence_paths(owners: &[EvidenceOwner]) -> Vec<String> {
    owners
        .iter()
        .flat_map(|owner| &owner.references)
        .filter_map(|reference| reference.file_path.as_deref())
        .filter_map(normalize_path)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalize_path(path: &str) -> Option<String> {
    let path = Utf8Path::new(path);
    if path.is_absolute() {
        return None;
    }
    let mut normalized = Utf8PathBuf::new();
    for component in path.components() {
        match component {
            Utf8Component::Normal(value) => normalized.push(value),
            Utf8Component::CurDir => {}
            Utf8Component::ParentDir | Utf8Component::RootDir | Utf8Component::Prefix(_) => {
                return None
            }
        }
    }
    (!normalized.as_str().is_empty()).then(|| normalized.into_string())
}

fn reverify(
    repo: &Utf8Path,
    base: &str,
    head: &str,
    owner: &EvidenceOwner,
    reference: &IdeationEvidenceReference,
    old_path: &str,
    renamed_path: Option<&str>,
) -> anyhow::Result<EvidenceResult> {
    let current_path = renamed_path.unwrap_or(old_path);
    let old_file = git::file_at(repo, base, old_path)?;
    let current_file = git::file_at(repo, head, current_path)?;
    let old_line = reference
        .line
        .and_then(|line| line_text(old_file.as_deref(), line));
    let matches = old_line.map_or_else(Vec::new, |needle| {
        matching_lines(current_file.as_deref(), needle)
    });
    let (status, current_line, reason) = if old_line.is_none() {
        (
            "unverifiable",
            None,
            "recorded evidence line is absent at the pinned commit".to_string(),
        )
    } else if matches.len() == 1 {
        let line = matches[0];
        let moved = renamed_path.is_some() || Some(line) != reference.line;
        (
            if moved { "moved" } else { "verified" },
            Some(line),
            if moved {
                "cited line was found at a new location"
            } else {
                "cited line remains at its recorded location"
            }
            .to_string(),
        )
    } else if matches.is_empty() {
        (
            "vanished",
            None,
            "exact cited line is no longer present".to_string(),
        )
    } else {
        (
            "unverifiable",
            None,
            "cited line now has multiple exact matches".to_string(),
        )
    };
    Ok(EvidenceResult {
        owner_type: owner.owner_type,
        owner_id: owner.owner_id.clone(),
        evidence_reference_id: reference.reference_id.as_str().to_string(),
        path: old_path.to_string(),
        current_path: (current_path != old_path).then(|| current_path.to_string()),
        recorded_line: reference.line,
        current_line,
        status,
        reason,
    })
}

fn line_text(file: Option<&str>, line: u32) -> Option<&str> {
    let index = usize::try_from(line).ok()?.checked_sub(1)?;
    file?
        .lines()
        .nth(index)
        .filter(|text| !text.trim().is_empty())
}

fn matching_lines(file: Option<&str>, needle: &str) -> Vec<u32> {
    file.into_iter()
        .flat_map(str::lines)
        .enumerate()
        .filter(|(_, line)| *line == needle)
        .filter_map(|(index, _)| u32::try_from(index + 1).ok())
        .collect()
}
