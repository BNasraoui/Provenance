use serde::{Deserialize, Serialize};

use super::ids::{SchemaVersion, ScopeId, StableId};
use super::parsing::normalize_enum_value;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ServiceEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<ServiceTier>,
    #[serde(default, alias = "externalId", skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub status: ServiceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceBinding {
    pub schema_version: SchemaVersion,
    pub scope_id: ScopeId,
    pub id: StableId,
    #[serde(alias = "ruleId")]
    pub rule_id: StableId,
    #[serde(alias = "serviceId")]
    pub service_id: StableId,
    #[serde(alias = "bindingType")]
    pub binding_type: ServiceBindingType,
}

impl ServiceBinding {
    pub fn stable_id(
        rule_id: &StableId,
        service_id: &StableId,
        binding_type: ServiceBindingType,
    ) -> anyhow::Result<StableId> {
        let binding_type = match binding_type {
            ServiceBindingType::Enforces => "enforces",
            ServiceBindingType::Consumes => "consumes",
            ServiceBindingType::Monitors => "monitors",
        };
        StableId::new(format!(
            "service_binding_{}_{}_{}",
            rule_id.as_str(),
            service_id.as_str(),
            binding_type,
        ))
    }
}
