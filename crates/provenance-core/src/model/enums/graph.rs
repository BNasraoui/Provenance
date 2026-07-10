use serde::{Deserialize, Serialize};

use super::normalize_enum_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "requirement")]
    Requirement,
    #[serde(rename = "resolution")]
    Resolution,
    #[serde(rename = "rule")]
    Rule,
    #[serde(rename = "topic")]
    Topic,
    #[serde(rename = "question")]
    Question,
}

impl NodeType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "source" => Ok(Self::Source),
            "requirement" => Ok(Self::Requirement),
            "resolution" => Ok(Self::Resolution),
            "rule" => Ok(Self::Rule),
            "topic" => Ok(Self::Topic),
            "question" => Ok(Self::Question),
            _ => anyhow::bail!(
                "parent type must be source, requirement, resolution, rule, topic, or question"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    #[serde(rename = "references")]
    References,
    #[serde(rename = "refines_into")]
    RefinesInto,
    #[serde(rename = "depends_on")]
    DependsOn,
    #[serde(rename = "contradicts")]
    Contradicts,
    #[serde(rename = "supersedes")]
    Supersedes,
    #[serde(rename = "needs")]
    Needs,
    #[serde(rename = "resolves")]
    Resolves,
    #[serde(rename = "spawns")]
    Spawns,
    #[serde(rename = "produces")]
    Produces,
}

impl EdgeType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match value {
            "references" => Ok(Self::References),
            "refines_into" => Ok(Self::RefinesInto),
            "depends_on" => Ok(Self::DependsOn),
            "contradicts" => Ok(Self::Contradicts),
            "supersedes" => Ok(Self::Supersedes),
            "needs" => Ok(Self::Needs),
            "resolves" => Ok(Self::Resolves),
            "spawns" => Ok(Self::Spawns),
            "produces" => Ok(Self::Produces),
            _ => anyhow::bail!("edge type must be a supported provenance edge type"),
        }
    }
}
