use camino::Utf8PathBuf;
use provenance_core::StableId;

#[derive(Debug, Clone)]
pub struct EvidenceSite {
    pub owner: EvidenceOwner,
    pub ownership: RequirementOwnership,
    pub source_id: StableId,
    pub repository: Utf8PathBuf,
    pub source_revision: Option<String>,
    pub revision: String,
    pub reference_id: StableId,
    pub path: String,
    pub line: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct EvidenceOwner {
    pub kind: OwnerKind,
    pub id: StableId,
    pub title: Option<String>,
    pub ratified: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OwnerKind {
    Proposal,
    Contribution,
}

impl OwnerKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposal => "proposal",
            Self::Contribution => "contribution",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RequirementOwnership {
    Proposal {
        proposal_id: StableId,
    },
    TargetRequirement {
        proposal_id: StableId,
        requirement_id: StableId,
    },
    CanonicalRequirement {
        proposal_id: StableId,
        requirement_id: StableId,
        decision_id: StableId,
    },
    NotApplicable,
}

impl RequirementOwnership {
    pub const fn requirement_id(&self) -> Option<&StableId> {
        match self {
            Self::TargetRequirement { requirement_id, .. }
            | Self::CanonicalRequirement { requirement_id, .. } => Some(requirement_id),
            Self::Proposal { .. } | Self::NotApplicable => None,
        }
    }

    pub const fn proposal_id(&self) -> Option<&StableId> {
        match self {
            Self::Proposal { proposal_id }
            | Self::TargetRequirement { proposal_id, .. }
            | Self::CanonicalRequirement { proposal_id, .. } => Some(proposal_id),
            Self::NotApplicable => None,
        }
    }

    pub const fn canonical_decision_id(&self) -> Option<&StableId> {
        match self {
            Self::CanonicalRequirement { decision_id, .. } => Some(decision_id),
            Self::Proposal { .. } | Self::TargetRequirement { .. } | Self::NotApplicable => None,
        }
    }

    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Proposal { .. } => "proposal",
            Self::TargetRequirement { .. } => "target_requirement",
            Self::CanonicalRequirement { .. } => "canonical_requirement",
            Self::NotApplicable => "not_applicable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationOutcome {
    Verified { line: u32 },
    Moved { line: u32 },
    Vanished,
    Unverifiable { reason: &'static str },
}

#[derive(Debug, serde::Serialize)]
pub struct Report {
    pub diffs: Vec<DiffRange>,
    pub evidence: Vec<EvidenceResult>,
    pub contradictions: Vec<Contradiction>,
    pub diagnostics: Vec<String>,
    pub summary: Summary,
}

#[derive(Debug, serde::Serialize)]
pub struct DiffRange {
    pub base: String,
    pub head: String,
    pub changed_paths: usize,
    pub intersecting_paths: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct EvidenceResult {
    pub owner_type: &'static str,
    pub owner_id: String,
    pub evidence_reference_id: String,
    pub source_id: String,
    pub repository: String,
    pub source_revision: Option<String>,
    pub base_revision: String,
    pub head_revision: String,
    pub requirement_ownership: &'static str,
    pub requirement_id: Option<String>,
    pub canonical_decision_id: Option<String>,
    pub path: String,
    pub current_path: Option<String>,
    pub recorded_line: Option<u32>,
    pub current_line: Option<u32>,
    pub status: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct Contradiction {
    pub proposal_id: String,
    pub requirement_id: String,
    pub requirement: String,
    pub evidence_reference_id: String,
    pub reason: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct Summary {
    pub graph_evidence_paths: usize,
    pub intersecting_paths: usize,
    pub evidence_reverified: usize,
    pub moved: usize,
    pub vanished: usize,
    pub contradictions: usize,
}
