//! Apply domain config settings at startup

use anyhow::{Context, Result, anyhow};
use domain_config::{DomainConfigKey, DomainConfigRepo, registry};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{info, warn};

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
        let value = serde_json::from_str(value_str).unwrap_or_else(|err| {
            // If it fails, treat it as a bare string
            warn!(
                key = %key,
                value = %value_str,
                error = %err,
                "Failed to parse config value as JSON, treating as string"
            );
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

/// Environment variable prefix for domain config settings.
///
/// Each domain config key can be set via an environment variable with this prefix.
/// The key is converted from kebab-case to SCREAMING_SNAKE_CASE.
///
/// Example:
/// - Config key: `require-verified-customer-for-account`
/// - Env var: `LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT=false`
pub const DOMAIN_CONFIG_ENV_PREFIX: &str = "LANA_DOMAIN_CONFIG_";

/// Parse domain config settings from environment variables.
///
/// Scans all environment variables for those starting with `LANA_DOMAIN_CONFIG_`
/// and converts them to domain config settings.
///
/// Example:
/// ```text
/// LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT=false
/// LANA_DOMAIN_CONFIG_SOME_OTHER_KEY=42
/// ```
pub fn parse_from_env() -> Result<Vec<DomainConfigSetting>> {
    let mut settings = Vec::new();

    for (env_key, env_value) in std::env::vars() {
        if let Some(suffix) = env_key.strip_prefix(DOMAIN_CONFIG_ENV_PREFIX) {
            if suffix.is_empty() {
                continue;
            }

            // Convert SCREAMING_SNAKE_CASE to kebab-case
            let config_key = env_var_suffix_to_config_key(suffix);

            // Parse the value as JSON (same logic as CLI parsing)
            let value = serde_json::from_str(&env_value).unwrap_or_else(|err| {
                // If it fails, treat it as a bare string
                warn!(
                    env_var = %env_key,
                    value = %env_value,
                    error = %err,
                    "Failed to parse env var value as JSON, treating as string"
                );
                serde_json::Value::String(env_value.clone())
            });

            settings.push(DomainConfigSetting {
                key: config_key,
                value,
            });
        }
    }

    // Sort for consistent ordering
    settings.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(settings)
}

/// Convert an environment variable suffix from SCREAMING_SNAKE_CASE to kebab-case.
///
/// Example: `REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT` -> `require-verified-customer-for-account`
fn env_var_suffix_to_config_key(suffix: &str) -> String {
    suffix.to_lowercase().replace('_', "-")
}

/// Convert a config key from kebab-case to SCREAMING_SNAKE_CASE for env var naming.
///
/// Example: `require-verified-customer-for-account` -> `REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT`
#[cfg(test)]
fn config_key_to_env_var_suffix(key: &str) -> String {
    key.to_uppercase().replace('-', "_")
}

/// Get the full environment variable name for a given config key.
///
/// Example: `require-verified-customer-for-account` -> `LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT`
#[cfg(test)]
fn config_key_to_env_var(key: &str) -> String {
    format!("{}{}", DOMAIN_CONFIG_ENV_PREFIX, config_key_to_env_var_suffix(key))
}

/// Collect domain config settings from both environment variable and CLI arguments.
///
/// CLI arguments take precedence over environment variable settings when the same key
/// is specified in both places.
pub fn collect_settings(cli_settings: Vec<DomainConfigSetting>) -> Result<Vec<DomainConfigSetting>> {
    let env_settings = parse_from_env()?;

    if env_settings.is_empty() {
        return Ok(cli_settings);
    }

    // Use a HashMap to deduplicate, with CLI taking precedence
    let mut settings_map: HashMap<String, DomainConfigSetting> = HashMap::new();

    // First add env settings
    for setting in env_settings {
        settings_map.insert(setting.key.clone(), setting);
    }

    // Then add CLI settings (overwriting any duplicates from env)
    for setting in cli_settings {
        settings_map.insert(setting.key.clone(), setting);
    }

    // Convert back to Vec, maintaining a consistent order
    let mut settings: Vec<_> = settings_map.into_values().collect();
    settings.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(settings)
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

    #[test]
    fn env_var_suffix_to_config_key_simple() {
        assert_eq!(
            env_var_suffix_to_config_key("REQUIRE_VERIFIED_CUSTOMER"),
            "require-verified-customer"
        );
    }

    #[test]
    fn env_var_suffix_to_config_key_single_word() {
        assert_eq!(env_var_suffix_to_config_key("ENABLED"), "enabled");
    }

    #[test]
    fn config_key_to_env_var_suffix_simple() {
        assert_eq!(
            config_key_to_env_var_suffix("require-verified-customer"),
            "REQUIRE_VERIFIED_CUSTOMER"
        );
    }

    #[test]
    fn config_key_to_env_var_full() {
        assert_eq!(
            config_key_to_env_var("require-verified-customer"),
            "LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER"
        );
    }

    #[test]
    fn collect_settings_cli_takes_precedence() {
        let cli_settings = vec![
            DomainConfigSetting {
                key: "shared-key".to_string(),
                value: serde_json::json!("from-cli"),
            },
            DomainConfigSetting {
                key: "cli-only".to_string(),
                value: serde_json::json!(true),
            },
        ];

        // Simulate env settings by testing the merge logic directly
        let env_settings = vec![
            DomainConfigSetting {
                key: "shared-key".to_string(),
                value: serde_json::json!("from-env"),
            },
            DomainConfigSetting {
                key: "env-only".to_string(),
                value: serde_json::json!(false),
            },
        ];

        let mut settings_map: HashMap<String, DomainConfigSetting> = HashMap::new();
        for setting in env_settings {
            settings_map.insert(setting.key.clone(), setting);
        }
        for setting in cli_settings {
            settings_map.insert(setting.key.clone(), setting);
        }

        let mut result: Vec<_> = settings_map.into_values().collect();
        result.sort_by(|a, b| a.key.cmp(&b.key));

        assert_eq!(result.len(), 3);

        let shared = result.iter().find(|s| s.key == "shared-key").unwrap();
        assert_eq!(shared.value, serde_json::json!("from-cli")); // CLI takes precedence

        assert!(result.iter().any(|s| s.key == "cli-only"));
        assert!(result.iter().any(|s| s.key == "env-only"));
    }
}
