use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::CustodianConfigId;

#[derive(Debug, Serialize, Deserialize)]
pub struct KomainuConfig {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CustodianConfig {
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
        config: CustodianConfig,
        audit_info: AuditInfo,
    },
}
