use async_graphql::*;

use crate::primitives::*;

pub use admin_graphql_shared::credit::CollateralBase;
pub use lana_app::credit::Collateral as DomainCollateral;

#[derive(InputObject)]
pub struct CollateralUpdateInput {
    pub collateral_id: UUID,
    pub collateral: Satoshis,
    pub effective: Date,
}

#[derive(InputObject)]
pub struct CollateralRecordSentToLiquidationInput {
    pub collateral_id: UUID,
    pub amount: Satoshis,
}

#[derive(InputObject)]
pub struct CollateralRecordProceedsFromLiquidationInput {
    pub collateral_id: UUID,
    pub amount: UsdCents,
}
