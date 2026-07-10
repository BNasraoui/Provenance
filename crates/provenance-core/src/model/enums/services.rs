use serde::{Deserialize, Serialize};

use super::normalize_enum_value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceEnvironment {
    #[serde(rename = "production")]
    Production,
    #[serde(rename = "staging")]
    Staging,
    #[serde(rename = "development")]
    Development,
}

impl ServiceEnvironment {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "production" => Ok(Self::Production),
            "staging" => Ok(Self::Staging),
            "development" => Ok(Self::Development),
            _ => anyhow::bail!("service environment must be production, staging, or development"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceTier {
    #[serde(rename = "critical")]
    Critical,
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "internal")]
    Internal,
}

impl ServiceTier {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "critical" => Ok(Self::Critical),
            "standard" => Ok(Self::Standard),
            "internal" => Ok(Self::Internal),
            _ => anyhow::bail!("service tier must be critical, standard, or internal"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "deprecated")]
    Deprecated,
    #[serde(rename = "decommissioned")]
    Decommissioned,
}

impl ServiceStatus {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "active" => Ok(Self::Active),
            "deprecated" => Ok(Self::Deprecated),
            "decommissioned" => Ok(Self::Decommissioned),
            _ => anyhow::bail!("service status must be active, deprecated, or decommissioned"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceBindingType {
    #[serde(rename = "enforces")]
    Enforces,
    #[serde(rename = "consumes")]
    Consumes,
    #[serde(rename = "monitors")]
    Monitors,
}

impl ServiceBindingType {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        match normalize_enum_value(value).as_str() {
            "enforces" => Ok(Self::Enforces),
            "consumes" => Ok(Self::Consumes),
            "monitors" => Ok(Self::Monitors),
            _ => anyhow::bail!("service binding type must be enforces, consumes, or monitors"),
        }
    }
}
