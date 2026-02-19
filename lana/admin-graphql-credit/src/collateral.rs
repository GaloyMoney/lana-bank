use async_graphql::*;

use crate::primitives::*;
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

#[derive(SimpleObject, Clone)]
pub struct CollateralBase {
    id: ID,
    collateral_id: UUID,
    pub wallet_id: Option<UUID>,
    account_id: UUID,

    #[graphql(skip)]
    pub entity: Arc<DomainCollateral>,
}

impl From<DomainCollateral> for CollateralBase {
    fn from(collateral: DomainCollateral) -> Self {
        Self {
            id: collateral.id.to_global_id(),
            collateral_id: collateral.id.into(),
            wallet_id: collateral.custody_wallet_id.map(|id| id.into()),
            account_id: collateral.account_ids.collateral_account_id.into(),
            entity: Arc::new(collateral),
        }
    }
}
