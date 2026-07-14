use super::model::{
    EvidenceClassification, EvidenceOwner, EvidenceSite, OwnerKind, ProposalClassification,
    RequirementOwnership,
};
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use provenance_core::{
    CanonicalArtifactType, IdeationTargetType, PromotionDecision, PromotionState, ProposalCard,
    StableId,
};
use provenance_store::state_store::ScopeSnapshot;
use std::collections::{BTreeMap, BTreeSet};

pub struct SiteCollection {
    pub sites: Vec<EvidenceSite>,
    pub diagnostics: Vec<String>,
}

struct SiteContext<'a> {
    repository: &'a Utf8Path,
    base_override: Option<&'a str>,
    sources: BTreeMap<&'a str, &'a provenance_core::Source>,
    requirements: BTreeSet<&'a str>,
    canonical: BTreeMap<&'a str, Vec<(&'a StableId, &'a StableId)>>,
}

pub fn collect(
    snapshot: &ScopeSnapshot,
    repository: &Utf8Path,
    base_override: Option<&str>,
) -> SiteCollection {
    let sources = snapshot
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect();
    let requirements = snapshot
        .requirements
        .iter()
        .map(|requirement| requirement.id.as_str())
        .collect();
    let mut canonical: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for decision in snapshot
        .promotion_decisions
        .iter()
        .filter(|decision| decision.decision == PromotionDecision::Accepted)
    {
        if let Some((proposal, ownership)) = (|| {
            let artifact = decision.canonical_artifact.as_ref()?;
            (artifact.artifact_type == CanonicalArtifactType::Requirement).then_some((
                decision.proposal_id.as_str(),
                (&artifact.artifact_id, &decision.id),
            ))
        })() {
            canonical.entry(proposal).or_default().push(ownership);
        }
    }
    let mut collection = SiteCollection {
        sites: Vec::new(),
        diagnostics: Vec::new(),
    };
    let context = SiteContext {
        repository,
        base_override,
        sources,
        requirements,
        canonical,
    };
    collect_proposals(snapshot, &context, &mut collection);
    collect_contributions(snapshot, &context, &mut collection);
    collection
}

fn collect_proposals(
    snapshot: &ScopeSnapshot,
    context: &SiteContext<'_>,
    collection: &mut SiteCollection,
) {
    for proposal in &snapshot.proposals {
        if proposal.traceability.evidence_references.is_empty() {
            continue;
        }
        let Some(ownership) = proposal_ownership(proposal, context, &mut collection.diagnostics)
        else {
            continue;
        };
        let [source_id] = proposal.traceability.source_ids.as_slice() else {
            let diagnosis = if proposal.traceability.source_ids.is_empty() {
                "has no source"
            } else {
                "has ambiguous sources"
            };
            collection.diagnostics.push(format!(
                "proposal {} {diagnosis}; its evidence sites were rejected",
                proposal.id.as_str()
            ));
            continue;
        };
        let Some(source) = context.sources.get(source_id.as_str()) else {
            collection.diagnostics.push(format!(
                "proposal {} references missing source {}",
                proposal.id.as_str(),
                source_id.as_str()
            ));
            continue;
        };
        let Some(revision) = context.base_override.or(source.commit_pin.as_deref()) else {
            collection.diagnostics.push(format!(
                "proposal {} was not diffed because source {} has no commit pin",
                proposal.id.as_str(),
                source_id.as_str()
            ));
            continue;
        };
        let owner = EvidenceOwner {
            kind: OwnerKind::Proposal,
            id: proposal.id.clone(),
            title: Some(proposal.title.clone()),
            ratified: proposal.promotion_state == PromotionState::Accepted,
        };
        add_references(
            collection,
            &owner,
            &ownership,
            source_id,
            context.repository,
            source.commit_pin.as_deref(),
            revision,
            &proposal.traceability.evidence_references,
        );
    }
}

fn proposal_ownership(
    proposal: &ProposalCard,
    context: &SiteContext<'_>,
    diagnostics: &mut Vec<String>,
) -> Option<RequirementOwnership> {
    if matches!(
        ProposalClassification::classify(proposal.proposal_type),
        ProposalClassification::Ineligible
    ) {
        diagnostics.push(format!(
            "proposal {} is not a requirement candidate; its evidence sites were rejected",
            proposal.id.as_str()
        ));
        return None;
    }
    let canonical = context
        .canonical
        .get(proposal.id.as_str())
        .map(Vec::as_slice);
    if let Some([(requirement_id, decision_id)]) = canonical {
        if !context.requirements.contains(requirement_id.as_str()) {
            diagnostics.push(format!(
                "proposal {} names missing canonical requirement {}; its evidence sites were rejected",
                proposal.id.as_str(),
                requirement_id.as_str()
            ));
            return None;
        }
        return Some(RequirementOwnership::CanonicalRequirement {
            proposal_id: proposal.id.clone(),
            requirement_id: (*requirement_id).clone(),
            decision_id: (*decision_id).clone(),
        });
    }
    if canonical.is_some() {
        diagnostics.push(format!(
            "proposal {} has ambiguous canonical requirements; its evidence sites were rejected",
            proposal.id.as_str()
        ));
        return None;
    }
    let Some(ownership) =
        RequirementOwnership::for_target(&proposal.id, &proposal.traceability.target)
    else {
        diagnostics.push(format!(
            "proposal {} does not target a requirement; its evidence sites were rejected",
            proposal.id.as_str()
        ));
        return None;
    };
    let requirement_id = ownership
        .requirement_id()
        .expect("target requirement ownership has a requirement");
    if !context.requirements.contains(requirement_id.as_str()) {
        diagnostics.push(format!(
            "proposal {} targets missing requirement {}; its evidence sites were rejected",
            proposal.id.as_str(),
            requirement_id.as_str()
        ));
        return None;
    }
    Some(ownership)
}

fn collect_contributions(
    snapshot: &ScopeSnapshot,
    context: &SiteContext<'_>,
    collection: &mut SiteCollection,
) {
    for contribution in &snapshot.contributions {
        if contribution.evidence_references.is_empty() {
            continue;
        }
        if contribution.target.artifact_type != IdeationTargetType::Source {
            collection.diagnostics.push(format!(
                "contribution {} has no explicit source ownership; its evidence sites were rejected",
                contribution.id.as_str()
            ));
            continue;
        }
        let source_id = &contribution.target.artifact_id;
        let Some(source) = context.sources.get(source_id.as_str()) else {
            collection.diagnostics.push(format!(
                "contribution {} references missing source {}",
                contribution.id.as_str(),
                source_id.as_str()
            ));
            continue;
        };
        let Some(revision) = context.base_override.or(source.commit_pin.as_deref()) else {
            collection.diagnostics.push(format!(
                "contribution {} was not diffed because source {} has no commit pin",
                contribution.id.as_str(),
                source_id.as_str()
            ));
            continue;
        };
        let owner = EvidenceOwner {
            kind: OwnerKind::Contribution,
            id: contribution.id.clone(),
            title: None,
            ratified: false,
        };
        add_references(
            collection,
            &owner,
            &RequirementOwnership::NotApplicable,
            source_id,
            context.repository,
            source.commit_pin.as_deref(),
            revision,
            &contribution.evidence_references,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn add_references(
    collection: &mut SiteCollection,
    owner: &EvidenceOwner,
    ownership: &RequirementOwnership,
    source_id: &StableId,
    repository: &Utf8Path,
    source_revision: Option<&str>,
    revision: &str,
    references: &[provenance_core::IdeationEvidenceReference],
) {
    for reference in references {
        if matches!(
            EvidenceClassification::classify(reference),
            EvidenceClassification::Ineligible
        ) {
            collection.diagnostics.push(format!(
                "evidence {} is not artifact evidence and was rejected",
                reference.reference_id.as_str()
            ));
            continue;
        }
        let Some(path) = reference.file_path.as_deref().and_then(normalize_path) else {
            collection.diagnostics.push(format!(
                "evidence {} has no valid repository-relative path",
                reference.reference_id.as_str()
            ));
            continue;
        };
        collection.sites.push(EvidenceSite {
            owner: owner.clone(),
            ownership: ownership.clone(),
            source_id: source_id.clone(),
            repository: repository.to_path_buf(),
            source_revision: source_revision.map(str::to_string),
            revision: revision.to_string(),
            reference_id: reference.reference_id.clone(),
            path,
            line: reference.line,
        });
    }
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
                return None;
            }
        }
    }
    (!normalized.as_str().is_empty()).then(|| normalized.into_string())
}
