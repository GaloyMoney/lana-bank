use audit::AuditInfo;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::DepositConfigId;

use super::{error::DepositConfigError, DepositConfigValues};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositConfigId")]
pub enum DepositConfigEvent {
    Initialized {
        id: DepositConfigId,
        reference: String,
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
    pub reference: String,
    values: Option<DepositConfigValues>,
    pub(super) events: EntityEvents<DepositConfigEvent>,
}

impl DepositConfig {
    pub fn values(&self) -> Result<DepositConfigValues, DepositConfigError> {
        self.values
            .clone()
            .ok_or(DepositConfigError::ValuesNotConfigured)
    }

    pub fn update_values(&mut self, new_values: DepositConfigValues, audit_info: AuditInfo) {
        self.events.push(DepositConfigEvent::DepositConfigUpdated {
            values: new_values.clone(),
            audit_info,
        });
        self.values = Some(new_values);
    }
}

impl TryFromEvents<DepositConfigEvent> for DepositConfig {
    fn try_from_events(events: EntityEvents<DepositConfigEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DepositConfigBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositConfigEvent::Initialized { id, reference } => {
                    builder = builder
                        .id(*id)
                        .reference(reference.to_string())
                        .values(None)
                }
                DepositConfigEvent::DepositConfigUpdated { values, .. } => {
                    builder = builder.values(Some(values.clone()))
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
    pub(super) reference: String,
}

impl NewDepositConfig {
    pub fn builder() -> NewDepositConfigBuilder {
        NewDepositConfigBuilder::default()
    }
}

impl IntoEvents<DepositConfigEvent> for NewDepositConfig {
    fn into_events(self) -> EntityEvents<DepositConfigEvent> {
        EntityEvents::init(
            self.id,
            [DepositConfigEvent::Initialized {
                id: self.id,
                reference: self.reference,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use audit::{AuditEntryId, AuditInfo};
    use chart_of_accounts::ChartId;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    #[test]
    fn test_create_and_update_new_deposit_config() {
        let id = DepositConfigId::new();

        let new_chart = NewDepositConfig::builder()
            .id(id)
            .reference("ref-01".to_string())
            .build()
            .unwrap();

        let events = new_chart.into_events();
        let mut deposit_config = DepositConfig::try_from_events(events).unwrap();
        assert_eq!(deposit_config.id, id);
        assert!(deposit_config.values().is_err());

        let new_values = DepositConfigValues::builder()
            .chart_of_accounts_id(ChartId::new())
            .chart_of_accounts_deposit_accounts_parent_code("11".parse().unwrap())
            .chart_of_accounts_omnibus_parent_code("12".parse().unwrap())
            .build()
            .unwrap();
        deposit_config.update_values(new_values, dummy_audit_info());
        assert!(deposit_config.values().is_ok());
    }
}
