use async_graphql::*;

use crate::{
    graphql::{accounting::LedgerAccount, credit_facility::CreditFacility, loader::LanaDataLoader},
    primitives::*,
};
pub use lana_app::credit::Collateral as DomainCollateral;

use super::Liquidation;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Collateral {
    id: ID,
    collateral_id: UUID,
    pub(crate) wallet_id: Option<UUID>,
    account_id: UUID,

    #[graphql(skip)]
    pub entity: Arc<DomainCollateral>,
}

impl From<DomainCollateral> for Collateral {
    fn from(collateral: DomainCollateral) -> Self {
        Self {
            id: collateral.id.to_global_id(),
            collateral_id: collateral.id.into(),
            wallet_id: collateral.custody_wallet_id.map(|id| id.into()),
            account_id: collateral.account_id.into(),
            entity: Arc::new(collateral),
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
    async fn credit_facility(&self, ctx: &Context<'_>) -> Result<CreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.entity.credit_facility_id)
            .await?
            .expect("Credit facility not found");
        Ok(facility)
    }

    async fn liquidation(&self, ctx: &Context<'_>) -> Result<Option<Liquidation>> {
        let liquidation_id = match self.entity.active_liquidation_id() {
            Some(id) => id,
            None => return Ok(None),
        };
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let liquidation = loader.load_one(liquidation_id).await?;
        Ok(liquidation)
    }
}
