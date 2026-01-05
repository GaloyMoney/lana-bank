use domain_config::{
    ConfigSpec, ConfigType as DomainConfigType, DomainConfigError, DomainConfigs, ValueKind,
};

use crate::notification::{NotificationFromEmailConfigSpec, NotificationFromNameConfigSpec};

#[derive(Debug, Clone)]
pub struct ExposedConfigItem {
    pub key: String,
    pub config_type: DomainConfigType,
    pub value: serde_json::Value,
    pub is_set: bool,
}

fn exposed_item<C: ConfigSpec>(value: serde_json::Value, is_set: bool) -> ExposedConfigItem {
    ExposedConfigItem {
        key: C::KEY.to_string(),
        config_type: <C::Kind as ValueKind>::TYPE,
        value,
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

macro_rules! exposed_configs {
    ($($C:ty),* $(,)?) => {
        async fn list_exposed_configs( domain_configs: &DomainConfigs,
        ) -> Result<Vec<ExposedConfigItem>, DomainConfigError> {
            let mut out = Vec::new();
            $(
                out.push(list_one::<$C>(domain_configs).await?);
            )*
            Ok(out)
        }

        async fn update_exposed_config(
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

#[derive(Clone)]
pub struct ExposedConfigs {
    domain_configs: DomainConfigs,
}

impl ExposedConfigs {
    pub fn new(domain_configs: &DomainConfigs) -> Self {
        Self {
            domain_configs: domain_configs.clone(),
        }
    }

    pub async fn list(&self) -> Result<Vec<ExposedConfigItem>, DomainConfigError> {
        list_exposed_configs(&self.domain_configs).await
    }

    pub async fn update(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<ExposedConfigItem, DomainConfigError> {
        update_exposed_config(&self.domain_configs, key, value).await
    }
}

// list all "exposed" configs defined in the system, here.
// This macro will generate methods to list all and
// update any, which can be used by our generic GraphQL schema
exposed_configs!(
    NotificationFromEmailConfigSpec,
    NotificationFromNameConfigSpec,
);
