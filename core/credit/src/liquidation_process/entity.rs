use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationProcessId")]
pub enum LiquidationProcessEvent {
    Initialized {
        id: LiquidationProcessId,
        tx_id: LedgerTxId,
        obligation_id: ObligationId,
        credit_facility_id: CreditFacilityId,
        in_liquidation_account_id: CalaAccountId,
        initial_amount: UsdCents,
        effective: chrono::NaiveDate,
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
    pub tx_id: LedgerTxId,
    pub obligation_id: ObligationId,
    pub credit_facility_id: CreditFacilityId,
    pub in_liquidation_account_id: CalaAccountId,
    pub initial_amount: UsdCents,
    pub effective: chrono::NaiveDate,
    events: EntityEvents<LiquidationProcessEvent>,
}

impl TryFromEvents<LiquidationProcessEvent> for LiquidationProcess {
    fn try_from_events(
        events: EntityEvents<LiquidationProcessEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = LiquidationProcessBuilder::default();
        for event in events.iter_all() {
            match event {
                LiquidationProcessEvent::Initialized {
                    id,
                    tx_id,
                    obligation_id,
                    credit_facility_id,
                    in_liquidation_account_id,
                    initial_amount,
                    effective,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .tx_id(*tx_id)
                        .obligation_id(*obligation_id)
                        .credit_facility_id(*credit_facility_id)
                        .in_liquidation_account_id(*in_liquidation_account_id)
                        .initial_amount(*initial_amount)
                        .effective(*effective)
                }
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
    pub(crate) tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(crate) obligation_id: ObligationId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) in_liquidation_account_id: CalaAccountId,
    pub(super) initial_amount: UsdCents,
    pub(super) effective: chrono::NaiveDate,
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
                tx_id: self.tx_id,
                obligation_id: self.obligation_id,
                credit_facility_id: self.credit_facility_id,
                in_liquidation_account_id: self.in_liquidation_account_id,
                initial_amount: self.initial_amount,
                effective: self.effective,
                audit_info: self.audit_info,
            }],
        )
    }
}
