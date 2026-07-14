use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ScopeId(String);

impl ScopeId {
    pub fn new(value: impl Into<String>) -> anyhow::Result<Self> {
        let value = value.into();
        if value.is_empty()
            || !value
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
        {
            anyhow::bail!("scope id must use lowercase ASCII letters, digits, '_' or '-'");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ScopeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value.clone()).map_err(|err| {
            D::Error::custom(format!(
                "scope id '{value}' is invalid: {err}; repair hint: correct this value where it is stored (state shard or input JSON)"
            ))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct StableId(String);

impl StableId {
    pub fn new(value: impl Into<String>) -> anyhow::Result<Self> {
        let value = value.into();
        if value.is_empty()
            || !value
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
        {
            anyhow::bail!("stable id must use lowercase ASCII letters, digits, '_' or '-'");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for StableId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value.clone()).map_err(|err| {
            D::Error::custom(format!(
                "stable id '{value}' is invalid: {err}; repair hint: correct this value where it is stored (state shard or input JSON)"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ScopeId, StableId};

    #[test]
    fn stable_id_deserialize_error_names_invalid_value_and_repair_hint() {
        let err = serde_json::from_str::<StableId>(r#""source/codebase""#).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("source/codebase"));
        assert!(message.contains("stable id"));
        assert!(message.contains("correct this value where it is stored"));
        assert!(message.contains("state shard or input JSON"));
    }

    #[test]
    fn scope_id_deserialize_error_names_invalid_value_and_repair_hint() {
        let err = serde_json::from_str::<ScopeId>(r#""Default""#).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("Default"));
        assert!(message.contains("scope id"));
        assert!(message.contains("correct this value where it is stored"));
        assert!(message.contains("state shard or input JSON"));
    }
}
