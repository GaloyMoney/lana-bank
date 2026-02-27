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
        key: &EncryptionKey,
        new_config: CustodianConfig,
    ) -> Result<Idempotent<()>, CustodianError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustodianEvent::ConfigUpdated { encrypted_custodian_config }
                if encrypted_custodian_config.matches_key(key)
                    && key.decrypt_json::<CustodianConfig>(encrypted_custodian_config).ok().as_ref() == Some(&new_config),
            => CustodianEvent::ConfigUpdated { .. }
        );

        if !self.encrypted_custodian_config.matches_key(key) {
            return Err(CustodianError::StaleEncryptionKey);
        }

        let encrypted = new_config.encrypt(key);
        self.encrypted_custodian_config = encrypted.clone();

        self.events.push(CustodianEvent::ConfigUpdated {
            encrypted_custodian_config: encrypted,
        });

        Ok(Idempotent::Executed(()))
    }

    fn encrypted_config_for_key(&self, key: &EncryptionKey) -> Option<&Encrypted> {
        self.events.iter_all().rev().find_map(|event| match event {
            CustodianEvent::ConfigUpdated {
                encrypted_custodian_config,
            } if encrypted_custodian_config.matches_key(key) => Some(encrypted_custodian_config),
            _ => None,
        })
    }

    fn custodian_config(&self, key: &EncryptionKey) -> Result<CustodianConfig, CustodianError> {
        let encrypted = self
            .encrypted_config_for_key(key)
            .ok_or(CustodianError::StaleEncryptionKey)?;
        CustodianConfig::decrypt(key, encrypted)
    }

    pub fn rotate_encryption_key(
        &mut self,
        new_key: &EncryptionKey,
        deprecated_key: &EncryptionKey,
    ) -> Result<Idempotent<()>, CustodianError> {
        if self.encrypted_custodian_config.matches_key(new_key) {
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
