use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
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
        audit_info: AuditInfo,
    },
    ConfigUpdated {
        encrypted_custodian_config: Option<EncryptedCustodianConfig>,
        audit_info: AuditInfo,
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
        audit_info: AuditInfo,
    ) {
        let encrypted_config = config.encrypt(secret);
        self.encrypted_custodian_config = encrypted_config.clone();

        self.events.push(CustodianEvent::ConfigUpdated {
            encrypted_custodian_config: Some(encrypted_config),
            audit_info,
        });
    }

    #[allow(dead_code)]
    fn custodian_config(&self, key: EncryptionKey) -> CustodianConfig {
        let (encrypted_config, nonce) = &self.encrypted_custodian_config;
        CustodianConfig::decrypt(&key, encrypted_config, nonce)
    }

    pub fn rotate_encryption_key(
        &mut self,
        encryption_key: &EncryptionKey,
        deprecated_encryption_key: &DeprecatedEncryptionKey,
        audit_info: &AuditInfo,
    ) -> Result<(), CustodianError> {
        let encrypted_config = CustodianConfig::rotate_encryption_key(
            encryption_key,
            &self.encrypted_custodian_config,
            deprecated_encryption_key,
        );

        self.encrypted_custodian_config = encrypted_config.clone();
        self.events.push(CustodianEvent::ConfigUpdated {
            encrypted_custodian_config: Some(encrypted_config),
            audit_info: audit_info.clone(),
        });

        Ok(())
    }

    #[cfg(not(feature = "mock-custodian"))]
    pub async fn custodian_client(
        self,
        key: EncryptionKey,
    ) -> Result<Box<dyn CustodianClient>, CustodianClientError> {
        match self.custodian_config(key) {
            CustodianConfig::Komainu(config) => {
                Ok(Box::new(komainu::KomainuClient::new(config.into())))
            }
            CustodianConfig::Bitgo(config) => Ok(Box::new(
                bitgo::BitgoClient::new(config.into()).map_err(CustodianClientError::client)?,
            )),
        }
    }

    #[cfg(feature = "mock-custodian")]
    pub async fn custodian_client(
        self,
        key: EncryptionKey,
    ) -> Result<Box<dyn CustodianClient>, CustodianClientError> {
        match self.custodian_config(key) {
            CustodianConfig::Komainu(config) => {
                Ok(Box::new(komainu::KomainuClient::new(config.into())))
            }
            CustodianConfig::Bitgo(config) => Ok(Box::new(
                bitgo::BitgoClient::new(config.into()).map_err(CustodianClientError::client)?,
            )),
            CustodianConfig::Mock => Ok(Box::new(super::client::mock::CustodianMock)),
        }
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
    pub(super) audit_info: AuditInfo,
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
                    audit_info: self.audit_info.clone(),
                },
                CustodianEvent::ConfigUpdated {
                    encrypted_custodian_config: Some(self.encrypted_custodian_config),
                    audit_info: self.audit_info,
                },
            ],
        )
    }
}
