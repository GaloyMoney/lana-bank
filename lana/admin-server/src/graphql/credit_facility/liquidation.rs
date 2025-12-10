use std::sync::Arc;

use async_graphql::*;
use lana_app::credit::liquidation::{Liquidation as DomainLiquidation, LiquidationEvent};

use crate::primitives::*;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Liquidation {
    id: ID,
    credit_facility_id: UUID,
    receivable_account_id: UUID,
    expected_to_receive: UsdCents,
    sent_total: Satoshis,
    received_total: UsdCents,
    // We keep the domain entity to access events in complex resolvers
    #[graphql(skip)]
    entity: Arc<DomainLiquidation>,
}

impl From<DomainLiquidation> for Liquidation {
    fn from(entity: DomainLiquidation) -> Self {
        Self {
            id: ID(entity.id.to_string()),
            credit_facility_id: entity.credit_facility_id.into(),
            receivable_account_id: entity.receivable_account_id.into(),
            expected_to_receive: entity.expected_to_receive,
            sent_total: entity.sent_total,
            received_total: entity.received_total,
            entity: Arc::new(entity),
        }
    }
}

#[derive(SimpleObject)]
pub struct LiquidationCollateralSent {
    amount: Satoshis,
    ledger_tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct LiquidationPaymentReceived {
    amount: UsdCents,
    ledger_tx_id: UUID,
}

#[ComplexObject]
impl Liquidation {
    async fn sent_collateral(&self) -> Vec<LiquidationCollateralSent> {
        let mut res = Vec::new();
        for event in self.entity.events().iter_all() {
            if let LiquidationEvent::CollateralSentOut {
                amount,
                ledger_tx_id,
            } = event
            {
                res.push(LiquidationCollateralSent {
                    amount: *amount,
                    ledger_tx_id: (*ledger_tx_id).into(),
                });
            }
        }
        res
    }

    async fn received_payment(&self) -> Vec<LiquidationPaymentReceived> {
        let mut res = Vec::new();
        for event in self.entity.events().iter_all() {
            if let LiquidationEvent::RepaymentAmountReceived {
                amount,
                ledger_tx_id,
            } = event
            {
                res.push(LiquidationPaymentReceived {
                    amount: *amount,
                    ledger_tx_id: (*ledger_tx_id).into(),
                });
            }
        }
        res
    }
}
