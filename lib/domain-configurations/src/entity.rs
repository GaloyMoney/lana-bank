use chrono::{DateTime, Utc};
use derive_builder::Builder;
use es_entity::*;
use serde::{Deserialize, Serialize};

use crate::primitives::DomainConfigurationKey;

#[derive(EsEvent, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DomainConfigurationKey")]
pub enum DomainConfigurationEvent {
    Updated {
        key: DomainConfigurationKey,
        value: serde_json::Value,
        updated_by: String,
        updated_at: DateTime<Utc>,
        reason: Option<String>,
        correlation_id: Option<String>,
        diff: serde_json::Value,
        previous_value: Option<serde_json::Value>,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DomainConfiguration {
    pub key: DomainConfigurationKey,
    pub value: serde_json::Value,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
    events: EntityEvents<DomainConfigurationEvent>,
}

impl DomainConfiguration {
    pub fn events(&self) -> &EntityEvents<DomainConfigurationEvent> {
        &self.events
    }
}

impl TryFromEvents<DomainConfigurationEvent> for DomainConfiguration {
    fn try_from_events(events: EntityEvents<DomainConfigurationEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DomainConfigurationBuilder::default();

        for event in events.iter_all() {
            if let DomainConfigurationEvent::Updated {
                key,
                value,
                updated_by,
                updated_at,
                reason,
                correlation_id,
                ..
            } = event
            {
                builder = builder
                    .key(key.clone())
                    .value(value.clone())
                    .updated_by(updated_by.clone())
                    .updated_at(*updated_at)
                    .reason(reason.clone())
                    .correlation_id(correlation_id.clone());
            }
        }

        builder.events(events).build()
    }
}

#[derive(Builder)]
pub struct NewDomainConfiguration {
    #[builder(setter(into))]
    pub key: DomainConfigurationKey,
    pub value: serde_json::Value,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
    pub diff: serde_json::Value,
    pub previous_value: Option<serde_json::Value>,
}

impl NewDomainConfiguration {
    pub fn builder() -> NewDomainConfigurationBuilder {
        Default::default()
    }
}

impl IntoEvents<DomainConfigurationEvent> for NewDomainConfiguration {
    fn into_events(self) -> EntityEvents<DomainConfigurationEvent> {
        EntityEvents::init(
            self.key,
            [DomainConfigurationEvent::Updated {
                key: self.key,
                value: self.value,
                updated_by: self.updated_by,
                updated_at: self.updated_at,
                reason: self.reason,
                correlation_id: self.correlation_id,
                diff: self.diff,
                previous_value: self.previous_value,
            }],
        )
    }
}
