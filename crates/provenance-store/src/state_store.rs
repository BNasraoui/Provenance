mod ideation_writers;
mod rule_writers;
mod thread_writers;
mod writers;

use crate::{layout::ProvenanceLayout, shards};
use provenance_core::{
    CanonicalArtifact, ClaimChallenge, ConsensusFinding, ContestedClaim, Contribution,
    ContributionStance, Edge, EvidenceGap, IdeationEvidenceReference, IdeationTarget, Manifest,
    MaterialClaim, Message, MessageRole, MinorityObjection, PromotionActor, PromotionDecision,
    PromotionDecisionRecord, PromotionState, ProposalCard, ProposalTraceability, ProposalType,
    RequiredHumanDecision, Requirement, RequirementStatus, Resolution, ResolutionStatus, Rule,
    RuleModality, RuleSeverity, RuleStatus, RuleType, ScopeId, Source, SourceType, StableId,
    SuggestedArtifact, SuggestedArtifactChange, SynthesisPacket, Thread, ThreadParent,
    UncertaintyRating, UnsupportedRecommendation, UnsupportedSpeculation,
};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct StateStore {
    pub(crate) layout: ProvenanceLayout,
}

pub struct CreateSourceInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    pub source_type: SourceType,
    pub url: Option<String>,
    pub reference: Option<String>,
}

pub struct CreateRequirementInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub statement: String,
    pub description: Option<String>,
    pub status: RequirementStatus,
}

pub struct AddSourceReferenceInput {
    pub scope_id: ScopeId,
    pub source_id: StableId,
    pub requirement_id: StableId,
    pub clause: Option<String>,
}

pub struct CreateResolutionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub title: String,
    pub requirement_id: Option<StableId>,
    pub position: String,
    pub rationale: String,
    pub status: ResolutionStatus,
    pub context: Option<String>,
    pub enforcement: Option<String>,
    pub confidence: Option<f64>,
}

pub struct CreateRuleInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub rule_code: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub requirement_id: Option<StableId>,
    pub resolution_id: Option<StableId>,
    pub statement: String,
    pub status: RuleStatus,
    pub severity: RuleSeverity,
    pub rule_type: Option<RuleType>,
    pub modality: Option<RuleModality>,
    pub confidence: Option<f64>,
    pub extraction_method: Option<String>,
    pub source_document: Option<String>,
    pub source_section: Option<String>,
}

pub struct PostMessageInput {
    pub scope_id: ScopeId,
    pub parent: ThreadParent,
    pub role: MessageRole,
    pub body: String,
}

pub struct CreateContributionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub target: IdeationTarget,
    pub participant_slot: String,
    pub stance: ContributionStance,
    pub strongest_finding: String,
    pub evidence_references: Vec<IdeationEvidenceReference>,
    pub material_claims: Vec<MaterialClaim>,
    pub risks: Vec<String>,
    pub objections: Vec<String>,
    pub challenges: Vec<ClaimChallenge>,
    pub suggested_artifact_changes: Vec<SuggestedArtifactChange>,
    pub unsupported_recommendations: Vec<UnsupportedRecommendation>,
    pub uncertainty: UncertaintyRating,
    pub open_questions: Vec<String>,
}

pub struct CreateSynthesisPacketInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub target: IdeationTarget,
    pub summary: String,
    pub consensus: Vec<ConsensusFinding>,
    pub contested_claims: Vec<ContestedClaim>,
    pub minority_objections: Vec<MinorityObjection>,
    pub evidence_gaps: Vec<EvidenceGap>,
    pub unsupported_speculation: Vec<UnsupportedSpeculation>,
    pub open_questions: Vec<String>,
    pub suggested_artifacts: Vec<SuggestedArtifact>,
    pub required_human_decisions: Vec<RequiredHumanDecision>,
}

pub struct CreateProposalCardInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub proposal_key: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub summary: String,
    pub traceability: ProposalTraceability,
    pub promotion_state: PromotionState,
    pub duplicate_of: Option<StableId>,
    pub superseded_by: Option<StableId>,
}

pub struct CreatePromotionDecisionInput {
    pub scope_id: ScopeId,
    pub id: StableId,
    pub proposal_id: StableId,
    pub decision: PromotionDecision,
    pub rationale: String,
    pub actor: PromotionActor,
    pub canonical_artifact: Option<CanonicalArtifact>,
}

#[derive(Debug, serde::Serialize)]
pub struct PostMessageResult {
    pub thread: Thread,
    pub message: Message,
}

impl StateStore {
    pub const fn new(layout: ProvenanceLayout) -> Self {
        Self { layout }
    }
    pub fn manifest(&self) -> anyhow::Result<Manifest> {
        Ok(serde_json::from_str(&std::fs::read_to_string(
            self.layout.manifest_path(),
        )?)?)
    }

    pub fn list_sources(&self, scope: &ScopeId) -> anyhow::Result<Vec<Source>> {
        read_jsonl(&shards::sources_path(&self.layout, scope))
    }
    pub fn list_requirements(&self, scope: &ScopeId) -> anyhow::Result<Vec<Requirement>> {
        read_jsonl(&shards::requirements_path(&self.layout, scope))
    }
    pub fn list_edges(&self) -> anyhow::Result<Vec<Edge>> {
        read_jsonl(&shards::edges_path(&self.layout))
    }
    pub fn list_resolutions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Resolution>> {
        read_jsonl(&shards::resolutions_path(&self.layout, scope))
    }
    pub fn list_rules(&self, scope: &ScopeId) -> anyhow::Result<Vec<Rule>> {
        read_jsonl(&shards::rules_path(&self.layout, scope))
    }
    pub fn list_threads(&self, scope: &ScopeId) -> anyhow::Result<Vec<Thread>> {
        read_jsonl(&shards::threads_path(&self.layout, scope))
    }
    pub fn list_messages(&self, scope: &ScopeId) -> anyhow::Result<Vec<Message>> {
        read_jsonl(&shards::messages_path(&self.layout, scope))
    }
    pub fn list_contributions(&self, scope: &ScopeId) -> anyhow::Result<Vec<Contribution>> {
        read_jsonl(&shards::contributions_path(&self.layout, scope))
    }
    pub fn list_synthesis_packets(&self, scope: &ScopeId) -> anyhow::Result<Vec<SynthesisPacket>> {
        read_jsonl(&shards::synthesis_packets_path(&self.layout, scope))
    }
    pub fn list_proposal_cards(&self, scope: &ScopeId) -> anyhow::Result<Vec<ProposalCard>> {
        read_jsonl(&shards::proposal_cards_path(&self.layout, scope))
    }
    pub fn list_promotion_decisions(
        &self,
        scope: &ScopeId,
    ) -> anyhow::Result<Vec<PromotionDecisionRecord>> {
        read_jsonl(&shards::promotion_decisions_path(&self.layout, scope))
    }
}

pub(crate) fn serde_name<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_value(value)?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("expected string enum serialization"))?
        .to_string())
}

fn read_jsonl<T: DeserializeOwned>(path: &camino::Utf8Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use provenance_core::{
        EdgeType, IdeationTargetType, IdentityType, Manifest, RepoPathPrefix, UncertaintyLevel,
    };

    #[test]
    fn source_requirement_records_are_written_deterministically() {
        let dir = tempfile::tempdir().unwrap();
        let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
        std::fs::write(
            layout.manifest_path(),
            serde_json::to_string(&Manifest::default_with_scope(
                ScopeId::new("default").unwrap(),
                RepoPathPrefix::new("."),
            ))
            .unwrap(),
        )
        .unwrap();
        let store = StateStore::new(layout);
        let scope = ScopeId::new("default").unwrap();
        store
            .create_source(CreateSourceInput {
                scope_id: scope.clone(),
                id: StableId::new("source_schads").unwrap(),
                name: "SCHADS Award".into(),
                source_type: SourceType::Policy,
                url: None,
                reference: None,
            })
            .unwrap();
        store
            .create_requirement(CreateRequirementInput {
                scope_id: scope.clone(),
                id: StableId::new("req_overtime").unwrap(),
                statement: "Overtime".into(),
                description: None,
                status: RequirementStatus::Active,
            })
            .unwrap();
        store
            .add_source_reference(AddSourceReferenceInput {
                scope_id: scope.clone(),
                source_id: StableId::new("source_schads").unwrap(),
                requirement_id: StableId::new("req_overtime").unwrap(),
                clause: None,
            })
            .unwrap();
        assert_eq!(
            store.list_sources(&scope).unwrap()[0].id.as_str(),
            "source_schads"
        );
        assert_eq!(
            store.list_edges().unwrap()[0].edge_type,
            EdgeType::References
        );
    }

    #[test]
    fn ideation_output_records_are_written_deterministically() {
        let dir = tempfile::tempdir().unwrap();
        let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
        std::fs::write(
            layout.manifest_path(),
            serde_json::to_string(&Manifest::default_with_scope(
                ScopeId::new("default").unwrap(),
                RepoPathPrefix::new("."),
            ))
            .unwrap(),
        )
        .unwrap();
        let store = StateStore::new(layout);
        let scope = ScopeId::new("default").unwrap();

        store
            .create_contribution(CreateContributionInput {
                scope_id: scope.clone(),
                id: StableId::new("contrib_b").unwrap(),
                target: IdeationTarget {
                    artifact_type: IdeationTargetType::Requirement,
                    artifact_id: StableId::new("req_overtime").unwrap(),
                },
                participant_slot: "reviewer".into(),
                stance: ContributionStance::Support,
                strongest_finding: "Supported by evidence".into(),
                evidence_references: Vec::new(),
                material_claims: Vec::new(),
                risks: Vec::new(),
                objections: Vec::new(),
                challenges: Vec::new(),
                suggested_artifact_changes: Vec::new(),
                unsupported_recommendations: Vec::new(),
                uncertainty: UncertaintyRating {
                    level: UncertaintyLevel::Low,
                    rationale: "Direct evidence".into(),
                },
                open_questions: Vec::new(),
            })
            .unwrap();
        store
            .create_contribution(CreateContributionInput {
                scope_id: scope.clone(),
                id: StableId::new("contrib_a").unwrap(),
                target: IdeationTarget {
                    artifact_type: IdeationTargetType::Requirement,
                    artifact_id: StableId::new("req_overtime").unwrap(),
                },
                participant_slot: "refuter".into(),
                stance: ContributionStance::NeedsMoreEvidence,
                strongest_finding: "Needs more evidence".into(),
                evidence_references: Vec::new(),
                material_claims: Vec::new(),
                risks: Vec::new(),
                objections: Vec::new(),
                challenges: Vec::new(),
                suggested_artifact_changes: Vec::new(),
                unsupported_recommendations: Vec::new(),
                uncertainty: UncertaintyRating {
                    level: UncertaintyLevel::High,
                    rationale: "Missing source".into(),
                },
                open_questions: Vec::new(),
            })
            .unwrap();

        assert_eq!(
            store.list_contributions(&scope).unwrap()[0].id.as_str(),
            "contrib_a"
        );
    }

    #[test]
    fn proposal_state_requires_duplicate_or_superseded_link() {
        let dir = tempfile::tempdir().unwrap();
        let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
        std::fs::write(
            layout.manifest_path(),
            serde_json::to_string(&Manifest::default_with_scope(
                ScopeId::new("default").unwrap(),
                RepoPathPrefix::new("."),
            ))
            .unwrap(),
        )
        .unwrap();
        let store = StateStore::new(layout);
        let scope = ScopeId::new("default").unwrap();

        let err = store
            .create_proposal_card(CreateProposalCardInput {
                scope_id: scope,
                id: StableId::new("proposal_duplicate").unwrap(),
                proposal_key: "duplicate".into(),
                proposal_type: ProposalType::RequirementCandidate,
                title: "Duplicate proposal".into(),
                summary: "This should point at the original proposal.".into(),
                traceability: ProposalTraceability {
                    target: IdeationTarget {
                        artifact_type: IdeationTargetType::Requirement,
                        artifact_id: StableId::new("req_overtime").unwrap(),
                    },
                    source_ids: Vec::new(),
                    evidence_references: Vec::new(),
                    supporting_claim_ids: Vec::new(),
                },
                promotion_state: PromotionState::Duplicate,
                duplicate_of: None,
                superseded_by: None,
            })
            .unwrap_err();

        assert!(err
            .to_string()
            .contains("duplicate proposals must set duplicate_of"));
    }

    #[test]
    fn promotion_decision_updates_proposal_state() {
        let dir = tempfile::tempdir().unwrap();
        let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let layout = ProvenanceLayout::new(root);
        std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
        std::fs::write(
            layout.manifest_path(),
            serde_json::to_string(&Manifest::default_with_scope(
                ScopeId::new("default").unwrap(),
                RepoPathPrefix::new("."),
            ))
            .unwrap(),
        )
        .unwrap();
        let store = StateStore::new(layout);
        let scope = ScopeId::new("default").unwrap();

        store
            .create_proposal_card(CreateProposalCardInput {
                scope_id: scope.clone(),
                id: StableId::new("proposal_overtime").unwrap(),
                proposal_key: "overtime".into(),
                proposal_type: ProposalType::RequirementCandidate,
                title: "Clarify overtime".into(),
                summary: "Clarify the overtime requirement.".into(),
                traceability: ProposalTraceability {
                    target: IdeationTarget {
                        artifact_type: IdeationTargetType::Requirement,
                        artifact_id: StableId::new("req_overtime").unwrap(),
                    },
                    source_ids: Vec::new(),
                    evidence_references: Vec::new(),
                    supporting_claim_ids: Vec::new(),
                },
                promotion_state: PromotionState::Proposed,
                duplicate_of: None,
                superseded_by: None,
            })
            .unwrap();
        store
            .create_promotion_decision(CreatePromotionDecisionInput {
                scope_id: scope.clone(),
                id: StableId::new("decision_overtime").unwrap(),
                proposal_id: StableId::new("proposal_overtime").unwrap(),
                decision: PromotionDecision::Accepted,
                rationale: "Approved by human review.".into(),
                actor: PromotionActor {
                    identity_type: IdentityType::Human,
                    id: "ben".into(),
                    name: None,
                },
                canonical_artifact: None,
            })
            .unwrap();

        assert_eq!(
            store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
            PromotionState::Accepted
        );
    }
}
