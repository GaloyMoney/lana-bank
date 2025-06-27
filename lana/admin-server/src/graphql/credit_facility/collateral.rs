use async_graphql::*;

use crate::primitives::*;
pub use lana_app::credit::Collateral as DomainCollateral;

#[derive(SimpleObject, Clone)]
pub struct Collateral {
    id: ID,
    collateral_id: UUID,
    pub(crate) wallet_id: Option<UUID>,
}

impl From<DomainCollateral> for Collateral {
    fn from(collateral: DomainCollateral) -> Self {
        Self {
            id: collateral.id.to_global_id(),
            collateral_id: collateral.id.into(),
            wallet_id: collateral.wallet_id.map(|id| id.into()),
        }
    }
}
