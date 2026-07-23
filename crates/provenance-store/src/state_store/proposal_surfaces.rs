use provenance_core::{
    ArtifactLinkTargetType, IdeationTarget, IdeationTargetType, PromotionState, ProposalCard,
    ScopeId, Topic,
};
use serde::Serialize;

use super::StateStore;

#[derive(Debug, Clone)]
pub struct ProposalDemand {
    changed_paths: Vec<String>,
    targets: Vec<IdeationTarget>,
}

impl ProposalDemand {
    pub fn for_changed_paths<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(paths.into_iter().map(Into::into).collect(), Vec::new())
    }

    pub fn for_target(target: IdeationTarget) -> Self {
        Self::new(Vec::new(), vec![target])
    }

    pub fn for_topic(topic: &Topic) -> Self {
        let mut targets = vec![
            IdeationTarget {
                artifact_type: IdeationTargetType::Topic,
                artifact_id: topic.id.clone(),
            },
            IdeationTarget {
                artifact_type: IdeationTargetType::Requirement,
                artifact_id: topic.requirement_id.clone(),
            },
        ];
        targets.extend(topic.links.iter().map(|link| IdeationTarget {
            artifact_type: match link.target_type {
                ArtifactLinkTargetType::Source => IdeationTargetType::Source,
                ArtifactLinkTargetType::Requirement => IdeationTargetType::Requirement,
                ArtifactLinkTargetType::Resolution => IdeationTargetType::Resolution,
                ArtifactLinkTargetType::Rule => IdeationTargetType::Rule,
            },
            artifact_id: link.target_id.clone(),
        }));
        Self::new(Vec::new(), targets)
    }

    pub fn new(mut changed_paths: Vec<String>, mut targets: Vec<IdeationTarget>) -> Self {
        changed_paths.sort();
        changed_paths.dedup();
        targets.sort_by(|left, right| {
            target_sort_key(left)
                .cmp(&target_sort_key(right))
                .then_with(|| left.artifact_id.as_str().cmp(right.artifact_id.as_str()))
        });
        targets.dedup();
        Self {
            changed_paths,
            targets,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "trigger", rename_all = "snake_case")]
pub enum ProposalSurfaceReason {
    EvidenceSite { path: String },
    Territory { target: IdeationTarget },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SurfacedProposal {
    pub proposal: ProposalCard,
    pub reasons: Vec<ProposalSurfaceReason>,
}

impl StateStore {
    pub fn surface_proposals(
        &self,
        scope: &ScopeId,
        demand: &ProposalDemand,
    ) -> anyhow::Result<Vec<SurfacedProposal>> {
        anyhow::ensure!(
            !demand.changed_paths.is_empty() || !demand.targets.is_empty(),
            "proposal demand must include at least one changed path or territory target"
        );

        Ok(self
            .list_proposal_cards(scope)?
            .into_iter()
            .filter(|proposal| {
                matches!(
                    proposal.promotion_state,
                    PromotionState::Proposed | PromotionState::Asserted
                )
            })
            .filter_map(|proposal| {
                let reasons = matching_reasons(&proposal, demand);
                (!reasons.is_empty()).then_some(SurfacedProposal { proposal, reasons })
            })
            .collect())
    }
}

fn matching_reasons(
    proposal: &ProposalCard,
    demand: &ProposalDemand,
) -> Vec<ProposalSurfaceReason> {
    let mut reasons = Vec::new();
    for path in &demand.changed_paths {
        if proposal
            .traceability
            .evidence_references
            .iter()
            .any(|reference| reference.file_path.as_deref() == Some(path.as_str()))
        {
            reasons.push(ProposalSurfaceReason::EvidenceSite { path: path.clone() });
        }
    }
    for target in &demand.targets {
        if proposal.traceability.target == *target {
            reasons.push(ProposalSurfaceReason::Territory {
                target: target.clone(),
            });
        }
    }
    reasons
}

const fn target_sort_key(target: &IdeationTarget) -> u8 {
    match target.artifact_type {
        IdeationTargetType::Source => 0,
        IdeationTargetType::Requirement => 1,
        IdeationTargetType::Resolution => 2,
        IdeationTargetType::Rule => 3,
        IdeationTargetType::Topic => 4,
        IdeationTargetType::Question => 5,
        IdeationTargetType::Domain => 6,
    }
}
