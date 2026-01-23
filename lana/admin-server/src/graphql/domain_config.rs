use async_graphql::*;

use domain_config::{ConfigType as DomainConfigType, DomainConfig as DomainConfigEntity};

use crate::{graphql::primitives::Json, primitives::*};

pub use domain_config::DomainConfigsByKeyCursor;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Bool,
    String,
    Int,
    Uint,
    Decimal,
    Timezone,
    Time,
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
            DomainConfigType::Timezone => ConfigType::Timezone,
            DomainConfigType::Time => ConfigType::Time,
            DomainConfigType::Complex => ConfigType::Complex,
        }
    }
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct DomainConfig {
    id: ID,
    domain_config_id: UUID,
    config_type: ConfigType,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainConfigEntity>,
}

impl From<DomainConfigEntity> for DomainConfig {
    fn from(config: DomainConfigEntity) -> Self {
        Self {
            id: config.id.to_global_id(),
            domain_config_id: UUID::from(config.id),
            config_type: config.config_type.into(),
            entity: Arc::new(config),
        }
    }
}

#[ComplexObject]
impl DomainConfig {
    async fn key(&self) -> &str {
        self.entity.key.as_str()
    }

    async fn value(&self) -> Json {
        Json::from(self.entity.current_json_value().clone())
    }
}

#[derive(InputObject)]
pub struct DomainConfigUpdateInput {
    pub domain_config_id: UUID,
    pub value: Json,
}
crate::mutation_payload! { DomainConfigUpdatePayload, domain_config: DomainConfig }
