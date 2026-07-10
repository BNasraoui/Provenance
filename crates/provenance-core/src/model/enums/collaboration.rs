use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "archived")]
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

impl MessageRole {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match value {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            _ => anyhow::bail!("role must be user, assistant, or system"),
        }
    }
}
