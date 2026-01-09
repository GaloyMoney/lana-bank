use async_graphql::*;

use domain_config::{
    ConfigType as DomainConfigType, DomainConfig,
};

use crate::{graphql::primitives::Json, primitives::*};

pub use domain_config::DomainConfigsByKeyCursor;

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

#[derive(Clone)]
pub struct ExposedConfig {
    pub(crate) entity: Arc<DomainConfig>,
}

impl From<DomainConfig> for ExposedConfig {
    fn from(config: DomainConfig) -> Self {
        Self {
            entity: Arc::new(config),
        }
    }
}

#[Object]
impl ExposedConfig {
    async fn id(&self) -> ID {
        self.entity.id.to_global_id()
    }

    async fn exposed_config_id(&self) -> UUID {
        UUID::from(self.entity.id)
    }

    async fn key(&self) -> &str {
        self.entity.key.as_str()
    }

    async fn config_type(&self) -> ConfigType {
        self.entity.config_type.into()
    }

    async fn value(&self) -> Json {
        Json::from(self.entity.current_json_value().clone())
    }
}

#[derive(InputObject)]
pub struct ExposedConfigUpdateInput {
    pub exposed_config_id: UUID,
    pub value: Json,
}
crate::mutation_payload! { ExposedConfigUpdatePayload, exposed_config: ExposedConfig }
