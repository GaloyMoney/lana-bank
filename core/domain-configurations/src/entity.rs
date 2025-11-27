use audit::AuditInfo;
use chrono::{DateTime, Utc};
use es_entity::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::primitives::DomainConfigurationKey;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DomainConfigurationKey")]
pub enum DomainConfigurationEvent {
    Updated {
        key: DomainConfigurationKey,
        new_value: Value,
        updated_by: String,
        updated_at: DateTime<Utc>,
        reason: Option<String>,
        correlation_id: Option<String>,
        diff_fields_csv: String,
        previous_value: Option<Value>,
    },
}

#[derive(EsEntity, Debug, Clone)]
pub struct DomainConfiguration {
    pub key: DomainConfigurationKey,
    pub value: Value,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,

    events: EntityEvents<DomainConfigurationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfigurationRecord<T> {
    pub value: T,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
}

impl DomainConfiguration {
    pub(super) fn apply_update(
        &mut self,
        audit_info: AuditInfo,
        updated_at: DateTime<Utc>,
        new_value: Value,
        reason: Option<String>,
        correlation_id: Option<String>,
    ) {
        let diff_fields_csv = diff_field_names(&self.value, &new_value);

        let event = DomainConfigurationEvent::Updated {
            key: self.key.clone(),
            new_value: new_value.clone(),
            updated_by: audit_info.sub,
            updated_at,
            reason: reason.clone(),
            correlation_id: correlation_id.clone(),
            diff_fields_csv,
            previous_value: Some(self.value.clone()),
        };

        self.value = new_value;
        self.updated_by = audit_info.sub;
        self.updated_at = updated_at;
        self.reason = reason;
        self.correlation_id = correlation_id;

        self.events.push(event);
    }
}

fn diff_field_names(prev: &Value, next: &Value) -> String {
    match (prev, next) {
        (Value::Object(prev_obj), Value::Object(next_obj)) => {
            let mut fields = Vec::new();
            for (key, next_val) in next_obj {
                if prev_obj.get(key) != Some(next_val) {
                    fields.push(key.clone());
                }
            }
            fields.join(",")
        }
        (_, Value::Object(next_obj)) => next_obj.keys().cloned().collect::<Vec<_>>().join(","),
        _ => String::new(),
    }
}

impl TryFromEvents<DomainConfigurationEvent> for DomainConfiguration {
    fn try_from_events(
        events: EntityEvents<DomainConfigurationEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut key = None;
        let mut value = None;
        let mut updated_by = None;
        let mut updated_at = None;
        let mut reason = None;
        let mut correlation_id = None;

        for event in events.iter_all() {
            match event {
                DomainConfigurationEvent::Updated {
                    key: event_key,
                    new_value,
                    updated_by: event_updated_by,
                    updated_at: event_updated_at,
                    reason: event_reason,
                    correlation_id: event_correlation_id,
                    ..
                } => {
                    key = Some(event_key.clone());
                    value = Some(new_value.clone());
                    updated_by = Some(event_updated_by.clone());
                    updated_at = Some(*event_updated_at);
                    reason = event_reason.clone();
                    correlation_id = event_correlation_id.clone();
                }
            }
        }

        let entity = DomainConfiguration {
            key: key.ok_or(EsEntityError::MissingField("key".to_owned()))?,
            value: value.ok_or(EsEntityError::MissingField("value".to_owned()))?,
            updated_by: updated_by
                .ok_or(EsEntityError::MissingField("updated_by".to_owned()))?,
            updated_at: updated_at
                .ok_or(EsEntityError::MissingField("updated_at".to_owned()))?,
            reason,
            correlation_id,
            events,
        };
        Ok(entity)
    }
}

#[derive(Debug)]
pub(super) struct NewDomainConfiguration {
    key: DomainConfigurationKey,
    value: Value,
    updated_by: String,
    updated_at: DateTime<Utc>,
    reason: Option<String>,
    correlation_id: Option<String>,
}

impl NewDomainConfiguration {
    pub(super) fn new(
        key: DomainConfigurationKey,
        audit_info: AuditInfo,
        updated_at: DateTime<Utc>,
        value: Value,
        reason: Option<String>,
        correlation_id: Option<String>,
    ) -> Self {
        Self {
            key,
            value,
            updated_by: audit_info.sub,
            updated_at,
            reason,
            correlation_id,
        }
    }
}

impl IntoEvents<DomainConfigurationEvent> for NewDomainConfiguration {
    fn into_events(self) -> EntityEvents<DomainConfigurationEvent> {
        EntityEvents::init(
            self.key.clone(),
            [DomainConfigurationEvent::Updated {
                key: self.key.clone(),
                new_value: self.value.clone(),
                updated_by: self.updated_by.clone(),
                updated_at: self.updated_at,
                reason: self.reason.clone(),
                correlation_id: self.correlation_id.clone(),
                diff_fields_csv: diff_field_names(&Value::Null, &self.value),
                previous_value: None,
            }],
        )
    }
}
