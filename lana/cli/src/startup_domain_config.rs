//! Apply domain config settings at startup

use anyhow::{Context, Result, anyhow};
use domain_config::{DomainConfigKey, DomainConfigRepo, registry};
use sqlx::PgPool;
use std::collections::HashMap;
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

/// Environment variable name for domain config settings.
pub const DOMAIN_CONFIG_ENV_VAR: &str = "LANA_DOMAIN_CONFIG";

/// Parse domain config settings from an environment variable.
///
/// The environment variable should contain comma-separated key=value pairs.
/// Example: `key1=true,key2=42,key3="string value"`
///
/// For values containing commas, use JSON array/object syntax or quote the entire value.
pub fn parse_from_env() -> Result<Vec<DomainConfigSetting>> {
    let env_value = match std::env::var(DOMAIN_CONFIG_ENV_VAR) {
        Ok(val) if !val.is_empty() => val,
        _ => return Ok(Vec::new()),
    };

    parse_comma_separated(&env_value)
        .with_context(|| format!("Failed to parse {} environment variable", DOMAIN_CONFIG_ENV_VAR))
}

/// Parse comma-separated key=value pairs.
///
/// Handles commas inside JSON values (objects/arrays) by tracking bracket depth.
fn parse_comma_separated(input: &str) -> Result<Vec<DomainConfigSetting>> {
    let mut settings = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                current.push(ch);
                escape_next = true;
            }
            '"' => {
                in_string = !in_string;
                current.push(ch);
            }
            '{' | '[' if !in_string => {
                bracket_depth += 1;
                current.push(ch);
            }
            '}' | ']' if !in_string => {
                bracket_depth -= 1;
                current.push(ch);
            }
            ',' if !in_string && bracket_depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    settings.push(DomainConfigSetting::parse(trimmed)?);
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Don't forget the last segment
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        settings.push(DomainConfigSetting::parse(trimmed)?);
    }

    Ok(settings)
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
    fn parse_comma_separated_simple() {
        let settings = parse_comma_separated("key1=true,key2=42,key3=hello").unwrap();
        assert_eq!(settings.len(), 3);
        assert_eq!(settings[0].key, "key1");
        assert_eq!(settings[0].value, serde_json::json!(true));
        assert_eq!(settings[1].key, "key2");
        assert_eq!(settings[1].value, serde_json::json!(42));
        assert_eq!(settings[2].key, "key3");
        assert_eq!(settings[2].value, serde_json::json!("hello"));
    }

    #[test]
    fn parse_comma_separated_with_spaces() {
        let settings = parse_comma_separated("key1=true , key2=false").unwrap();
        assert_eq!(settings.len(), 2);
        assert_eq!(settings[0].key, "key1");
        assert_eq!(settings[1].key, "key2");
    }

    #[test]
    fn parse_comma_separated_with_json_object() {
        let settings =
            parse_comma_separated(r#"key1=true,complex={"a":1,"b":2},key2=false"#).unwrap();
        assert_eq!(settings.len(), 3);
        assert_eq!(settings[0].key, "key1");
        assert_eq!(settings[1].key, "complex");
        assert_eq!(settings[1].value, serde_json::json!({"a": 1, "b": 2}));
        assert_eq!(settings[2].key, "key2");
    }

    #[test]
    fn parse_comma_separated_with_json_array() {
        let settings = parse_comma_separated(r#"key1=true,items=[1,2,3],key2=false"#).unwrap();
        assert_eq!(settings.len(), 3);
        assert_eq!(settings[1].key, "items");
        assert_eq!(settings[1].value, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn parse_comma_separated_with_quoted_string_containing_comma() {
        let settings = parse_comma_separated(r#"key1=true,msg="hello, world",key2=false"#).unwrap();
        assert_eq!(settings.len(), 3);
        assert_eq!(settings[1].key, "msg");
        assert_eq!(settings[1].value, serde_json::json!("hello, world"));
    }

    #[test]
    fn parse_comma_separated_empty() {
        let settings = parse_comma_separated("").unwrap();
        assert!(settings.is_empty());
    }

    #[test]
    fn parse_comma_separated_single() {
        let settings = parse_comma_separated("key=value").unwrap();
        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].key, "key");
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
