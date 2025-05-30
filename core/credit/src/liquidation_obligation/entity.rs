use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationObligationId")]
pub enum LiquidationObligationEvent {
    Initialized {
        id: LiquidationObligationId,
        parent_obligation_id: ObligationId,
        credit_facility_id: CreditFacilityId,
        tx_id: LedgerTxId,
        receivable_account_id: CalaAccountId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LiquidationObligation {
    pub id: LiquidationObligationId,
    pub parent_obligation_id: ObligationId,
    pub credit_facility_id: CreditFacilityId,
    pub tx_id: LedgerTxId,
    pub receivable_account_id: CalaAccountId,
    events: EntityEvents<LiquidationObligationEvent>,
}

impl TryFromEvents<LiquidationObligationEvent> for LiquidationObligation {
    fn try_from_events(
        events: EntityEvents<LiquidationObligationEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = LiquidationObligationBuilder::default();
        for event in events.iter_all() {
            match event {
                LiquidationObligationEvent::Initialized {
                    id,
                    parent_obligation_id,
                    credit_facility_id,
                    tx_id,
                    receivable_account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .parent_obligation_id(*parent_obligation_id)
                        .credit_facility_id(*credit_facility_id)
                        .tx_id(*tx_id)
                        .receivable_account_id(*receivable_account_id)
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLiquidationObligation {
    #[builder(setter(into))]
    pub(crate) id: LiquidationObligationId,
    #[builder(setter(into))]
    pub(crate) parent_obligation_id: ObligationId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(super) receivable_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewLiquidationObligation {
    pub fn builder() -> NewLiquidationObligationBuilder {
        NewLiquidationObligationBuilder::default()
    }
}

impl IntoEvents<LiquidationObligationEvent> for NewLiquidationObligation {
    fn into_events(self) -> EntityEvents<LiquidationObligationEvent> {
        EntityEvents::init(
            self.id,
            [LiquidationObligationEvent::Initialized {
                id: self.id,
                parent_obligation_id: self.parent_obligation_id,
                credit_facility_id: self.credit_facility_id,
                tx_id: self.tx_id,
                receivable_account_id: self.receivable_account_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
