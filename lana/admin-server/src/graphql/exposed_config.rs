use async_graphql::connection::CursorType;
use async_graphql::*;
use serde::{Deserialize, Serialize};

use domain_config::{
    ConfigType as DomainConfigType, DomainConfig,
    DomainConfigsByKeyCursor as DomainConfigsByKeyCursorDomain,
};

use crate::{graphql::primitives::Json, primitives::*};

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct DomainConfigsByKeyCursor(DomainConfigsByKeyCursorDomain);

impl DomainConfigsByKeyCursor {
    pub(crate) fn into_domain(self) -> DomainConfigsByKeyCursorDomain {
        self.0
    }
}

impl From<&DomainConfig> for DomainConfigsByKeyCursor {
    fn from(config: &DomainConfig) -> Self {
        Self(DomainConfigsByKeyCursorDomain::from(config))
    }
}

impl CursorType for DomainConfigsByKeyCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{Engine as _, engine::general_purpose};
        let json = serde_json::to_string(&self).expect("could not serialize cursor");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

#[derive(InputObject)]
pub struct ExposedConfigUpdateInput {
    pub exposed_config_id: UUID,
    pub value: Json,
}
crate::mutation_payload! { ExposedConfigUpdatePayload, exposed_config: ExposedConfig }
