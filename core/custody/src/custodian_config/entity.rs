use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::CustodianConfigId;

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Custodian {
    Manual,
    Komainu(KomainuConfig),
}

#[derive(EsEvent, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CustodianConfigId")]
pub enum CustodianConfigEvent {
    Initialized {
        id: CustodianConfigId,
        name: String,
        custodian: Custodian,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct CustodianConfig {
    pub id: CustodianConfigId,
    pub name: String,
    pub custodian: Custodian,
    events: EntityEvents<CustodianConfigEvent>,
}

impl TryFromEvents<CustodianConfigEvent> for CustodianConfig {
    fn try_from_events(events: EntityEvents<CustodianConfigEvent>) -> Result<Self, EsEntityError> {
        let mut builder = CustodianConfigBuilder::default();

        for event in events.iter_all() {
            match event {
                CustodianConfigEvent::Initialized {
                    id,
                    name,
                    custodian,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .name(name.clone())
                        .custodian(custodian.clone())
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCustodianConfig {
    #[builder(setter(into))]
    pub(super) id: CustodianConfigId,
    pub(super) name: String,
    pub(super) custodian: Custodian,
    pub(super) audit_info: AuditInfo,
}

impl NewCustodianConfig {
    pub fn builder() -> NewCustodianConfigBuilder {
        Default::default()
    }
}

impl IntoEvents<CustodianConfigEvent> for NewCustodianConfig {
    fn into_events(self) -> EntityEvents<CustodianConfigEvent> {
        EntityEvents::init(
            self.id,
            [CustodianConfigEvent::Initialized {
                id: self.id,
                name: self.name,
                custodian: self.custodian,
                audit_info: self.audit_info,
            }],
        )
    }
}
