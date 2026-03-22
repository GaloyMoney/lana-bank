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
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("domainConfigId".to_string())
)]
pub struct DomainConfig {
    domain_config_id: DomainConfigId,
    config_type: ConfigType,
    encrypted: bool,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainConfigEntity>,
}

impl From<DomainConfigEntity> for DomainConfig {
    fn from(config: DomainConfigEntity) -> Self {
        Self {
            domain_config_id: config.id,
            config_type: config.config_type.into(),
            encrypted: config.encrypted,
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
        match self.entity.current_stored_value() {
            Some(stored) => Json::from(stored.plain_or_null()),
            None => Json::from(serde_json::Value::Null),
        }
    }

    async fn is_set(&self) -> bool {
        self.entity.current_stored_value().is_some()
    }
}

#[derive(InputObject)]
pub struct DomainConfigUpdateInput {
    pub domain_config_id: DomainConfigId,
    pub value: Json,
}
crate::mutation_payload! { DomainConfigUpdatePayload, domain_config: DomainConfig }
