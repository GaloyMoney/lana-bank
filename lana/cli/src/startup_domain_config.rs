//! Apply domain config settings at startup

use anyhow::{Context, Result, anyhow};
use domain_config::{DomainConfigKey, DomainConfigRepo, registry};
use sqlx::PgPool;
use tracing::info;

/// A domain config setting parsed from CLI input.
#[derive(Debug, Clone)]
pub struct DomainConfigSetting {
    pub key: String,
    pub value: serde_json::Value,
}

impl DomainConfigSetting {
    /// Parse a "key=value" string into a DomainConfigSetting.
    ///
    /// Values are parsed as JSON:
    /// - `true` / `false` -> boolean
    /// - `123` -> number
    /// - `"string"` -> string (with quotes)
    /// - Bare strings without quotes -> treated as string
    pub fn parse(input: &str) -> Result<Self> {
        let (key, value_str) = input
            .split_once('=')
            .ok_or_else(|| anyhow!("Invalid format: expected 'key=value', got '{}'", input))?;

        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(anyhow!("Key cannot be empty"));
        }

        let value_str = value_str.trim();

        // Try to parse as JSON first
        let value = serde_json::from_str(value_str).unwrap_or_else(|_| {
            // If it fails, treat it as a bare string
            serde_json::Value::String(value_str.to_string())
        });

        Ok(Self { key, value })
    }
}

/// Apply domain config settings to the database.
pub async fn apply_settings(pool: &PgPool, settings: &[DomainConfigSetting]) -> Result<()> {
    let repo = DomainConfigRepo::new(pool);

    for setting in settings {
        let entry = registry::maybe_find_by_key(&setting.key).ok_or_else(|| {
            let available_keys = list_available_keys();
            anyhow!(
                "Unknown domain config key: '{}'\n\nAvailable keys:\n{}",
                setting.key,
                available_keys
            )
        })?;

        // Validate the value using the registry's validator
        (entry.validate_json)(&setting.value)
            .with_context(|| format!("Invalid value for '{}': {}", setting.key, setting.value))?;

        // Find and update the config
        let key: DomainConfigKey = setting.key.clone().into();
        let mut config = repo.find_by_key(key).await.with_context(|| {
            format!(
                "Failed to find config '{}' in database. Ensure seeding has completed.",
                setting.key
            )
        })?;

        let idempotent = config
            .apply_update_from_json(entry, setting.value.clone())
            .with_context(|| format!("Failed to apply update for '{}'", setting.key))?;

        if idempotent.did_execute() {
            repo.update(&mut config)
                .await
                .with_context(|| format!("Failed to save config '{}'", setting.key))?;
            info!(key = %setting.key, value = %setting.value, "Applied domain config setting");
        } else {
            info!(key = %setting.key, value = %setting.value, "Domain config setting already applied");
        }
    }

    Ok(())
}

/// List all available domain config keys for error messages.
pub fn list_available_keys() -> String {
    let mut keys: Vec<_> = registry::all_specs().map(|spec| spec.key).collect();
    keys.sort();
    keys.iter()
        .map(|k| format!("  - {}", k))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_true() {
        let setting = DomainConfigSetting::parse("my-key=true").unwrap();
        assert_eq!(setting.key, "my-key");
        assert_eq!(setting.value, serde_json::json!(true));
    }

    #[test]
    fn parse_boolean_false() {
        let setting = DomainConfigSetting::parse("my-key=false").unwrap();
        assert_eq!(setting.key, "my-key");
        assert_eq!(setting.value, serde_json::json!(false));
    }

    #[test]
    fn parse_number() {
        let setting = DomainConfigSetting::parse("max-limit=42").unwrap();
        assert_eq!(setting.key, "max-limit");
        assert_eq!(setting.value, serde_json::json!(42));
    }

    #[test]
    fn parse_quoted_string() {
        let setting = DomainConfigSetting::parse(r#"timezone="America/New_York""#).unwrap();
        assert_eq!(setting.key, "timezone");
        assert_eq!(setting.value, serde_json::json!("America/New_York"));
    }

    #[test]
    fn parse_bare_string() {
        let setting = DomainConfigSetting::parse("timezone=America/New_York").unwrap();
        assert_eq!(setting.key, "timezone");
        assert_eq!(setting.value, serde_json::json!("America/New_York"));
    }

    #[test]
    fn parse_with_spaces() {
        let setting = DomainConfigSetting::parse("  my-key  =  true  ").unwrap();
        assert_eq!(setting.key, "my-key");
        assert_eq!(setting.value, serde_json::json!(true));
    }

    #[test]
    fn parse_missing_equals() {
        let result = DomainConfigSetting::parse("no-equals-sign");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("key=value"));
    }

    #[test]
    fn parse_empty_key() {
        let result = DomainConfigSetting::parse("=value");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn parse_json_object() {
        let setting =
            DomainConfigSetting::parse(r#"complex-config={"enabled":true,"limit":10}"#).unwrap();
        assert_eq!(setting.key, "complex-config");
        assert_eq!(
            setting.value,
            serde_json::json!({"enabled": true, "limit": 10})
        );
    }
}
