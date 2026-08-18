use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

pub const SDK_PROTOCOL_VERSION: u32 = 3;

/// Language-neutral contract advertised by the Rust engine before SDK work.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EngineInfo {
    pub engine_version: String,
    pub protocol_version: u32,
    pub state_schema_version: u32,
    pub repository: Utf8PathBuf,
}
