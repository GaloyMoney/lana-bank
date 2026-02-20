use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use encryption::{Encrypted, EncryptionKey};

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
        encrypted_custodian_config: Encrypted,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Custodian {
    pub id: CustodianId,
    encrypted_custodian_config: Encrypted,
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
        new_config: CustodianConfig,
        key: &EncryptionKey,
    ) -> Idempotent<()> {
        let current_config = CustodianConfig::try_decrypt(key, &self.encrypted_custodian_config);
        if current_config.as_ref() == Some(&new_config) {
            return Idempotent::AlreadyApplied;
        }

        let encrypted = new_config.encrypt(key);
        self.encrypted_custodian_config = encrypted.clone();

        self.events.push(CustodianEvent::ConfigUpdated {
            encrypted_custodian_config: encrypted,
        });

        Idempotent::Executed(())
    }

    fn custodian_config(&self, key: &EncryptionKey) -> Result<CustodianConfig, CustodianError> {
        CustodianConfig::decrypt(key, &self.encrypted_custodian_config)
    }

    pub fn rotate_encryption_key(
        &mut self,
        new_key: &EncryptionKey,
        deprecated_key: &EncryptionKey,
    ) -> Result<Idempotent<()>, CustodianError> {
        if CustodianConfig::try_decrypt(new_key, &self.encrypted_custodian_config).is_some() {
            return Ok(Idempotent::AlreadyApplied);
        }

        let encrypted_config = CustodianConfig::rotate_encryption_key(
            new_key,
            deprecated_key,
            &self.encrypted_custodian_config,
        )?;

        self.encrypted_custodian_config = encrypted_config.clone();
        self.events.push(CustodianEvent::ConfigUpdated {
            encrypted_custodian_config: encrypted_config,
        });

        Ok(Idempotent::Executed(()))
    }

    #[record_error_severity]
    #[instrument(name = "custody.custodian_client", skip(self, key), fields(custodian_id = %self.id))]
    pub fn custodian_client(
        self,
        key: &EncryptionKey,
        provider_config: &CustodyProviderConfig,
    ) -> Result<Box<dyn CustodianClient>, CustodianClientError> {
        self.custodian_config(key)
            .map_err(CustodianClientError::client)?
            .custodian_client(provider_config)
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
                    builder = builder.encrypted_custodian_config(encrypted_custodian_config.clone())
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
    pub(super) encrypted_custodian_config: Encrypted,
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
        key: &EncryptionKey,
    ) -> &mut Self {
        self.encrypted_custodian_config = Some(custodian_config.encrypt(key));
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
                    encrypted_custodian_config: self.encrypted_custodian_config,
                },
            ],
        )
    }
}
