use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use es_entity::*;

use crate::primitives::CustodianId;

use super::client::{CustodianClient, error::CustodianClientError};
use super::{config::*, error::*};

#[derive(EsEvent, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CustodianId")]
pub enum CustodianEvent {
    Initialized {
        id: CustodianId,
        name: String,
        provider: String,
    },
    ConfigUpdated {
        encrypted_custodian_config: Option<EncryptedCustodianConfig>,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Custodian {
    pub id: CustodianId,
    encrypted_custodian_config: EncryptedCustodianConfig,
    pub name: String,
    pub(super) provider: String,
    events: EntityEvents<CustodianEvent>,
}

impl Custodian {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for Custodian")
    }

    pub fn update_custodian_config(
        &mut self,
        config: CustodianConfig,
        secret: &EncryptionKey,
    ) -> Idempotent<()> {
        let (current_cypher, current_nonce) = &self.encrypted_custodian_config;
        let current_config = CustodianConfig::decrypt(secret, current_cypher, current_nonce);
        if current_config == config {
            return Idempotent::AlreadyApplied;
        }

        let encrypted_config = config.encrypt(secret);
        self.encrypted_custodian_config = encrypted_config.clone();

        self.events.push(CustodianEvent::ConfigUpdated {
            encrypted_custodian_config: Some(encrypted_config),
        });

        Idempotent::Executed(())
    }

    fn custodian_config(&self, key: EncryptionKey) -> CustodianConfig {
        let (encrypted_config, nonce) = &self.encrypted_custodian_config;
        CustodianConfig::decrypt(&key, encrypted_config, nonce)
    }

    pub fn rotate_encryption_key(
        &mut self,
        encryption_key: &EncryptionKey,
        deprecated_encryption_key: &DeprecatedEncryptionKey,
    ) -> Result<Idempotent<()>, CustodianError> {
        let (current_cypher, current_nonce) = &self.encrypted_custodian_config;
        if CustodianConfig::try_decrypt(encryption_key, current_cypher, current_nonce).is_some() {
            return Ok(Idempotent::AlreadyApplied);
        }

        let encrypted_config = CustodianConfig::rotate_encryption_key(
            encryption_key,
            &self.encrypted_custodian_config,
            deprecated_encryption_key,
        );

        self.encrypted_custodian_config = encrypted_config.clone();
        self.events.push(CustodianEvent::ConfigUpdated {
            encrypted_custodian_config: Some(encrypted_config),
        });

        Ok(Idempotent::Executed(()))
    }

    #[record_error_severity]
    #[instrument(name = "custody.custodian_client", skip(self, key), fields(custodian_id = %self.id))]
    pub fn custodian_client(
        self,
        key: EncryptionKey,
        provider_config: &CustodyProviderConfig,
    ) -> Result<Box<dyn CustodianClient>, CustodianClientError> {
        self.custodian_config(key).custodian_client(provider_config)
    }
}

impl TryFromEvents<CustodianEvent> for Custodian {
    fn try_from_events(events: EntityEvents<CustodianEvent>) -> Result<Self, EsEntityError> {
        let mut builder = CustodianBuilder::default();

        for event in events.iter_all() {
            match event {
                CustodianEvent::Initialized {
                    id, name, provider, ..
                } => {
                    builder = builder
                        .id(*id)
                        .name(name.clone())
                        .provider(provider.clone())
                }
                CustodianEvent::ConfigUpdated {
                    encrypted_custodian_config,
                    ..
                } => {
                    if let Some(config) = encrypted_custodian_config {
                        builder = builder.encrypted_custodian_config(config.clone())
                    }
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Builder)]
pub struct NewCustodian {
    #[builder(setter(into))]
    pub(super) id: CustodianId,
    pub(super) name: String,
    pub(super) provider: String,
    #[builder(setter(custom))]
    pub(super) encrypted_custodian_config: EncryptedCustodianConfig,
}

impl NewCustodian {
    pub fn builder() -> NewCustodianBuilder {
        Default::default()
    }
}

impl NewCustodianBuilder {
    pub fn encrypted_custodian_config(
        &mut self,
        custodian_config: CustodianConfig,
        encryption_key: &EncryptionKey,
    ) -> &mut Self {
        self.encrypted_custodian_config = Some(custodian_config.encrypt(encryption_key));
        self
    }
}

impl IntoEvents<CustodianEvent> for NewCustodian {
    fn into_events(self) -> EntityEvents<CustodianEvent> {
        EntityEvents::init(
            self.id,
            [
                CustodianEvent::Initialized {
                    id: self.id,
                    name: self.name,
                    provider: self.provider,
                },
                CustodianEvent::ConfigUpdated {
                    encrypted_custodian_config: Some(self.encrypted_custodian_config),
                },
            ],
        )
    }
}
