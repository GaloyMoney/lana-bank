use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationProcessId")]
pub enum LiquidationProcessEvent {
    Initialized {
        id: LiquidationProcessId,
        parent_obligation_id: ObligationId,
        credit_facility_id: CreditFacilityId,
        audit_info: AuditInfo,
    },
    Completed {
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LiquidationProcess {
    pub id: LiquidationProcessId,
    events: EntityEvents<LiquidationProcessEvent>,
}

impl TryFromEvents<LiquidationProcessEvent> for LiquidationProcess {
    fn try_from_events(
        events: EntityEvents<LiquidationProcessEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = LiquidationProcessBuilder::default();
        for event in events.iter_all() {
            match event {
                LiquidationProcessEvent::Initialized { id, .. } => builder = builder.id(*id),
                LiquidationProcessEvent::Completed { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLiquidationProcess {
    #[builder(setter(into))]
    pub(crate) id: LiquidationProcessId,
    #[builder(setter(into))]
    pub(crate) parent_obligation_id: ObligationId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewLiquidationProcess {
    pub fn builder() -> NewLiquidationProcessBuilder {
        NewLiquidationProcessBuilder::default()
    }
}

impl IntoEvents<LiquidationProcessEvent> for NewLiquidationProcess {
    fn into_events(self) -> EntityEvents<LiquidationProcessEvent> {
        EntityEvents::init(
            self.id,
            [LiquidationProcessEvent::Initialized {
                id: self.id,
                parent_obligation_id: self.parent_obligation_id,
                credit_facility_id: self.credit_facility_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
