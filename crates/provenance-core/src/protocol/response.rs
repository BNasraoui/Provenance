use camino::Utf8PathBuf;
use serde::Serialize;

use crate::coverage::{EvidenceDiffSite, EvidenceDiffSummary};
use crate::model::{
    ImplementationBinding, RequirementReview, VerificationBinding, VerificationRun,
};

use super::{AffectedRule, GraphNode, Neighbor, TracedNode, SDK_PROTOCOL_VERSION};

/// The envelope every query primitive answers in.
///
/// The protocol version travels with the answer, so a caller holding a
/// recorded response can tell which contract produced it, and `operation`
/// names which primitive it came from.
#[derive(Debug, Clone, Serialize)]
pub struct QueryResponse<Result> {
    pub protocol_version: u32,
    pub operation: &'static str,
    #[serde(flatten)]
    pub result: Result,
}

impl<Result> QueryResponse<Result> {
    pub const fn new(operation: &'static str, result: Result) -> Self {
        Self {
            protocol_version: SDK_PROTOCOL_VERSION,
            operation,
            result,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GetResult {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<GraphNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub limit: usize,
    pub has_more: bool,
    pub nodes: Vec<GraphNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NeighborsResult {
    pub id: String,
    pub limit: usize,
    pub has_more: bool,
    pub neighbors: Vec<Neighbor>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceResult {
    pub id: String,
    pub max_depth: usize,
    pub limit: usize,
    pub has_more: bool,
    pub nodes: Vec<TracedNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactResult {
    pub id: String,
    pub limit: usize,
    pub has_more: bool,
    pub affected_rules: Vec<AffectedRule>,
}

/// What a commit range did to the code carrying a Rule's evidence.
#[derive(Debug, Clone, Serialize)]
pub struct StaleEvidence {
    pub base: String,
    pub head: String,
    pub sites: Vec<EvidenceDiffSite>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceResult {
    pub rule_id: String,
    pub limit: usize,
    pub has_more: bool,
    pub implementation_bindings: Vec<ImplementationBinding>,
    pub verification_bindings: Vec<VerificationBinding>,
    pub verification_runs: Vec<VerificationRun>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_verification_run: Option<VerificationRun>,
    pub review_required: bool,
    pub reviews: Vec<RequirementReview>,
    pub stale: Option<StaleEvidence>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StaleResult {
    pub base: String,
    pub head: String,
    pub files_changed: usize,
    pub summary: EvidenceDiffSummary,
    pub limit: usize,
    pub has_more: bool,
    pub sites: Vec<EvidenceDiffSite>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolveSymbolResult {
    pub file: Utf8PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub limit: usize,
    pub has_more: bool,
    pub rules: Vec<GraphNode>,
}
