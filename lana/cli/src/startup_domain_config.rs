//! Parse domain config settings from environment variables at startup

use anyhow::Result;
use tracing::info;

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

            // Parse the value as JSON, or treat as string if not valid JSON
            let value = serde_json::from_str(&env_value).unwrap_or_else(|_| {
                info!(
                    env_var = %env_key,
                    value = %env_value,
                    "Env var value is not valid JSON, treating as string"
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
        let env_var = format!(
            "{}{}",
            DOMAIN_CONFIG_ENV_PREFIX,
            key.to_uppercase().replace('-', "_")
        );
        assert_eq!(env_var, "LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER");
    }
}
