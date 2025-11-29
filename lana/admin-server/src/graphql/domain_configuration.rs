use async_graphql::{ErrorExtensions, InputObject, SimpleObject};
use chrono::{DateTime, Utc};

use lana_app::domain_configurations::ExampleConfig as DomainExampleConfig;

#[derive(SimpleObject, Clone)]
pub struct ExampleConfiguration {
    feature_enabled: bool,
    threshold: i32,
}

impl From<DomainExampleConfig> for ExampleConfiguration {
    fn from(value: DomainExampleConfig) -> Self {
        Self {
            feature_enabled: value.feature_enabled,
            threshold: value.threshold as i32,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ExampleConfigurationRecord {
    value: ExampleConfiguration,
    updated_by: String,
    updated_at: DateTime<Utc>,
    reason: Option<String>,
    correlation_id: Option<String>,
}

impl From<lana_app::domain_configurations::DomainConfigurationRecord<DomainExampleConfig>>
    for ExampleConfigurationRecord
{
    fn from(
        value: lana_app::domain_configurations::DomainConfigurationRecord<DomainExampleConfig>,
    ) -> Self {
        Self {
            value: ExampleConfiguration::from(value.value),
            updated_by: value.updated_by,
            updated_at: value.updated_at,
            reason: value.reason,
            correlation_id: value.correlation_id,
        }
    }
}

#[derive(InputObject)]
pub struct ExampleConfigurationSetInput {
    pub feature_enabled: bool,
    pub threshold: i32,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
}

crate::mutation_payload! { ExampleConfigurationSetPayload, example_configuration: ExampleConfigurationRecord }

pub fn map_domain_configuration_error(
    err: lana_app::domain_configurations::DomainConfigurationError,
) -> async_graphql::Error {
    match err {
        lana_app::domain_configurations::DomainConfigurationError::NotSet => {
            async_graphql::Error::new("Configuration not set").extend_with(|_, e| {
                e.set("code", "NOT_SET");
            })
        }
        lana_app::domain_configurations::DomainConfigurationError::Forbidden => {
            async_graphql::Error::new("Forbidden").extend_with(|_, e| {
                e.set("code", "FORBIDDEN");
            })
        }
        lana_app::domain_configurations::DomainConfigurationError::Invalid(msg) => {
            async_graphql::Error::new(msg).extend_with(|_, e| {
                e.set("code", "BAD_USER_INPUT");
            })
        }
        lana_app::domain_configurations::DomainConfigurationError::Sqlx(e) => {
            async_graphql::Error::new(e.to_string())
        }
        lana_app::domain_configurations::DomainConfigurationError::EsEntityError(e) => {
            async_graphql::Error::new(e.to_string())
        }
        lana_app::domain_configurations::DomainConfigurationError::Internal(e) => {
            async_graphql::Error::new(e.to_string())
        }
    }
}
