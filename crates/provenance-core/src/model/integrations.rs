use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use super::{SchemaVersion, ScopeId, StableId};

/// The lifecycle state of one callback-backed verification run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationRunStatus {
    Running,
    Passed,
    Failed,
}

impl VerificationRunStatus {
    pub fn parse_completion(value: &str) -> anyhow::Result<Self> {
        match value {
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            _ => anyhow::bail!("verification completion status must be passed or failed"),
        }
    }
}

/// Volatile evidence from one language-owned verification callback.
///
/// Runs live in Provenance's derived cache rather than canonical state: a
/// local test run must not dirty the repository. `rule_id` is the join back
/// to the canonical graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRun {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub rule_id: StableId,
    pub method: String,
    pub declared_by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<Utf8PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub status: VerificationRunStatus,
    pub started_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
