//! Apply domain config settings at startup via environment variables

use anyhow::{Context, Result, anyhow};
use domain_config::{DomainConfigKey, DomainConfigRepo, registry};
use sqlx::PgPool;
use tracing::{info, warn};

/// A domain config setting parsed from an environment variable.
#[derive(Debug, Clone)]
pub struct DomainConfigSetting {
    pub key: String,
    pub value: serde_json::Value,
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

            // Parse the value as JSON
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
fn list_available_keys() -> String {
    let mut keys: Vec<_> = registry::all_specs().map(|spec| spec.key).collect();
    keys.sort();
    keys.iter()
        .map(|k| format!("  - {}", k))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert an environment variable suffix from SCREAMING_SNAKE_CASE to kebab-case.
///
/// Example: `REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT` -> `require-verified-customer-for-account`
fn env_var_suffix_to_config_key(suffix: &str) -> String {
    suffix.to_lowercase().replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn config_key_to_env_var_suffix() {
        // Reverse conversion for documentation purposes
        let key = "require-verified-customer";
        let expected = "REQUIRE_VERIFIED_CUSTOMER";
        assert_eq!(key.to_uppercase().replace('-', "_"), expected);
    }

    #[test]
    fn config_key_to_env_var_full() {
        let key = "require-verified-customer";
        let env_var = format!("{}{}", DOMAIN_CONFIG_ENV_PREFIX, key.to_uppercase().replace('-', "_"));
        assert_eq!(env_var, "LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER");
    }
}
