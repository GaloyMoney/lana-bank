use audit::AuditInfo;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::DepositConfigId;

use super::DepositConfigValues;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositConfigId")]
pub enum DepositConfigEvent {
    Initialized {
        id: DepositConfigId,
    },
    DepositConfigUpdated {
        values: DepositConfigValues,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DepositConfig {
    pub id: DepositConfigId,
    pub values: DepositConfigValues,
    pub(super) events: EntityEvents<DepositConfigEvent>,
}

impl DepositConfig {
    pub fn update_values(&mut self, new_values: DepositConfigValues, audit_info: AuditInfo) {
        self.events.push(DepositConfigEvent::DepositConfigUpdated {
            values: new_values.clone(),
            audit_info,
        });
        self.values = new_values;
    }
}

impl TryFromEvents<DepositConfigEvent> for DepositConfig {
    fn try_from_events(events: EntityEvents<DepositConfigEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DepositConfigBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositConfigEvent::Initialized { id } => builder = builder.id(*id),
                DepositConfigEvent::DepositConfigUpdated { values, .. } => {
                    builder = builder.values(values.clone())
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDepositConfig {
    #[builder(setter(into))]
    pub(super) id: DepositConfigId,
}

impl NewDepositConfig {
    pub fn builder() -> NewDepositConfigBuilder {
        NewDepositConfigBuilder::default()
    }
}

impl IntoEvents<DepositConfigEvent> for NewDepositConfig {
    fn into_events(self) -> EntityEvents<DepositConfigEvent> {
        EntityEvents::init(self.id, [DepositConfigEvent::Initialized { id: self.id }])
    }
}
