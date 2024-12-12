use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::primitives::ChartOfAccountId;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartOfAccountId")]
pub enum ChartOfAccountEvent {
    Initialized {
        id: ChartOfAccountId,
        audit_info: AuditInfo,
    },
}
#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ChartOfAccount {
    pub id: ChartOfAccountId,
    pub(super) events: EntityEvents<ChartOfAccountEvent>,
}

impl TryFromEvents<ChartOfAccountEvent> for ChartOfAccount {
    fn try_from_events(events: EntityEvents<ChartOfAccountEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartOfAccountBuilder::default();
        for event in events.iter_all() {
            match event {
                ChartOfAccountEvent::Initialized { id, .. } => builder = builder.id(*id),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewChartOfAccount {
    #[builder(setter(into))]
    pub(super) id: ChartOfAccountId,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewChartOfAccount {
    pub fn builder() -> NewChartOfAccountBuilder {
        NewChartOfAccountBuilder::default()
    }
}

impl IntoEvents<ChartOfAccountEvent> for NewChartOfAccount {
    fn into_events(self) -> EntityEvents<ChartOfAccountEvent> {
        EntityEvents::init(
            self.id,
            [ChartOfAccountEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info,
            }],
        )
    }
}
