use async_graphql::*;

use crate::primitives::*;
pub use lana_app::credit::Liquidation as DomainLiquidation;

#[derive(Clone, SimpleObject)]
pub struct Liquidation {
    id: ID,
    liquidation_id: UUID,
    expected_to_receive: UsdCents,
    sent_total: Satoshis,
    received_total: UsdCents,
    created_at: Timestamp,
    completed: bool,
}

impl From<DomainLiquidation> for Liquidation {
    fn from(liquidation: DomainLiquidation) -> Self {
        Self {
            id: liquidation.id.to_global_id(),
            liquidation_id: UUID::from(liquidation.id),
            expected_to_receive: liquidation.expected_to_receive,
            sent_total: liquidation.sent_total,
            received_total: liquidation.received_total,
            created_at: liquidation.created_at().into(),
            completed: liquidation.is_completed(),
        }
    }
}
