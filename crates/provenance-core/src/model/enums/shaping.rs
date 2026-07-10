use serde::{Deserialize, Serialize};

use super::normalize_enum_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactLinkTargetType {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "requirement")]
    Requirement,
    #[serde(rename = "resolution")]
    Resolution,
    #[serde(rename = "rule")]
    Rule,
}

impl ArtifactLinkTargetType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "source" => Ok(Self::Source),
            "requirement" => Ok(Self::Requirement),
            "resolution" => Ok(Self::Resolution),
            "rule" => Ok(Self::Rule),
            _ => anyhow::bail!(
                "artifact link target type must be source, requirement, resolution, or rule"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopicStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "explored")]
    Explored,
    #[serde(rename = "closed")]
    Closed,
}

impl TopicStatus {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "open" => Ok(Self::Open),
            "explored" => Ok(Self::Explored),
            "closed" => Ok(Self::Closed),
            _ => anyhow::bail!("topic status must be open, explored, or closed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestionStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "blocked_on_human", alias = "blocked-on-human")]
    BlockedOnHuman,
    #[serde(rename = "answered")]
    Answered,
}

impl QuestionStatus {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "open" => Ok(Self::Open),
            "blocked_on_human" => Ok(Self::BlockedOnHuman),
            "answered" => Ok(Self::Answered),
            _ => anyhow::bail!("question status must be open, blocked_on_human, or answered"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionMethod {
    #[serde(rename = "grill")]
    Grill,
    #[serde(rename = "prototype")]
    Prototype,
    #[serde(rename = "research")]
    Research,
    #[serde(rename = "verify")]
    Verify,
    #[serde(rename = "task")]
    Task,
}

impl ResolutionMethod {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "grill" => Ok(Self::Grill),
            "prototype" => Ok(Self::Prototype),
            "research" => Ok(Self::Research),
            "verify" => Ok(Self::Verify),
            "task" => Ok(Self::Task),
            _ => anyhow::bail!(
                "resolution method must be grill, prototype, research, verify, or task"
            ),
        }
    }
}
