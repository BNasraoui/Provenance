use serde::{Deserialize, Serialize};

use crate::model::{
    SchemaVersion, ScopeId, ServiceBindingType, ServiceEnvironment, ServiceStatus, ServiceTier,
    StableId,
};

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
