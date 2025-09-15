use async_graphql::*;

use crate::{
    graphql::{accounting::LedgerAccount, loader::LanaDataLoader},
    primitives::*,
};
pub use lana_app::credit::Collateral as DomainCollateral;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Collateral {
    id: ID,
    collateral_id: UUID,
    pub(crate) wallet_id: Option<UUID>,
    account_id: UUID,
}

impl From<DomainCollateral> for Collateral {
    fn from(collateral: DomainCollateral) -> Self {
        Self {
            id: collateral.id.to_global_id(),
            collateral_id: collateral.id.into(),
            wallet_id: collateral.custody_wallet_id.map(|id| id.into()),
            account_id: collateral.account_id.into(),
        }
    }
}

#[ComplexObject]
impl Collateral {
    async fn account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(LedgerAccountId::from(self.account_id))
            .await?
            .expect("Collateral account not found");
        Ok(collateral)
    }
}
