use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::CustodianId;

use super::{custodian_config::*, error::*};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KomainuConfig {
    pub api_key: String,
    pub api_secret: String,
    pub testing_instance: bool,
    pub secret_key: String,
}

impl core::fmt::Debug for KomainuConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KomainuConfig")
            .field("api_key", &self.api_key)
            .field("api_secret", &"<redacted>")
            .field("testing_instance", &self.testing_instance)
            .field("secret_key", &"<redacted>")
            .finish()
    }
}

#[derive(EsEvent, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CustodianId")]
pub enum CustodianEvent {
    Initialized {
        id: CustodianId,
        name: String,
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
}

impl TryFromEvents<CustodianEvent> for Custodian {
    fn try_from_events(events: EntityEvents<CustodianEvent>) -> Result<Self, EsEntityError> {
        let mut builder = CustodianBuilder::default();

        for event in events.iter_all() {
            match event {
                CustodianEvent::Initialized { id, name, .. } => {
                    builder = builder.id(*id).name(name.clone())
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
