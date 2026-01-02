use async_graphql::*;

use domain_config::{
    ConfigSpec, ConfigType as DomainConfigType, DomainConfigError, DomainConfigs, ValueKind,
};

use lana_app::notification::{NotificationFromEmailConfigSpec, NotificationFromNameConfigSpec};

use crate::graphql::primitives::Json;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Bool,
    String,
    Int,
    Uint,
    Decimal,
    Complex,
}

impl From<DomainConfigType> for ConfigType {
    fn from(value: DomainConfigType) -> Self {
        match value {
            DomainConfigType::Bool => ConfigType::Bool,
            DomainConfigType::String => ConfigType::String,
            DomainConfigType::Int => ConfigType::Int,
            DomainConfigType::Uint => ConfigType::Uint,
            DomainConfigType::Decimal => ConfigType::Decimal,
            DomainConfigType::Complex => ConfigType::Complex,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ExposedConfigItem {
    pub key: String,
    pub config_type: ConfigType,
    pub value: Json,
    pub is_set: bool,
}

#[derive(InputObject)]
pub struct ExposedConfigUpdateInput {
    pub key: String,
    pub value: Json,
}
crate::mutation_payload! { ExposedConfigUpdatePayload, exposed_config: ExposedConfigItem }

macro_rules! exposed_configs {
    ($($C:ty),* $(,)?) => {
        pub async fn list_exposed_configs(
            domain_configs: &DomainConfigs,
        ) -> Result<Vec<ExposedConfigItem>, DomainConfigError> {
            let mut out = Vec::new();
            $(
                out.push(list_one::<$C>(domain_configs).await?);
            )*
            Ok(out)
        }

        pub async fn update_exposed_config(
            domain_configs: &DomainConfigs,
            key: String,
            value: serde_json::Value,
        ) -> Result<ExposedConfigItem, DomainConfigError> {
            match key.as_str() {
                $(
                    k if k == <$C as ConfigSpec>::KEY.as_str() => {
                        update_one::<$C>(domain_configs, value).await
                    }
                )*
                _ => Err(DomainConfigError::InvalidType(format!(
                    "Unknown exposed config key: {key}"
                ))),
            }
        }
    };
}

fn exposed_item<C: ConfigSpec>(value: serde_json::Value, is_set: bool) -> ExposedConfigItem {
    ExposedConfigItem {
        key: C::KEY.to_string(),
        config_type: <C::Kind as ValueKind>::TYPE.into(),
        value: value.into(),
        is_set,
    }
}

async fn list_one<C: ConfigSpec>(
    domain_configs: &DomainConfigs,
) -> Result<ExposedConfigItem, DomainConfigError> {
    let (value, is_set) = match domain_configs.get::<C>().await {
        Ok(value) => (<C::Kind as ValueKind>::encode(&value)?, true),
        Err(DomainConfigError::NotConfigured) => (serde_json::Value::Null, false),
        Err(err) => return Err(err),
    };

    Ok(exposed_item::<C>(value, is_set))
}

async fn update_one<C: ConfigSpec>(
    domain_configs: &DomainConfigs,
    value: serde_json::Value,
) -> Result<ExposedConfigItem, DomainConfigError>
where
    <C::Kind as ValueKind>::Value: Clone,
{
    let typed_value = <C::Kind as ValueKind>::decode(value)?;
    C::validate(&typed_value)?;

    let encoded = <C::Kind as ValueKind>::encode(&typed_value)?;
    domain_configs.upsert::<C>(typed_value).await?;

    Ok(exposed_item::<C>(encoded, true))
}

exposed_configs!(
    NotificationFromEmailConfigSpec,
    NotificationFromNameConfigSpec,
);
