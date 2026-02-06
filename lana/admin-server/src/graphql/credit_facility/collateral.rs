use async_graphql::*;

use crate::{
    graphql::{accounting::LedgerAccount, credit_facility::CreditFacility, loader::LanaDataLoader},
    primitives::*,
};
pub use lana_app::credit::Collateral as DomainCollateral;

#[derive(InputObject)]
pub struct CollateralUpdateInput {
    pub collateral_id: UUID,
    pub collateral: Satoshis,
    pub effective: Date,
}
crate::mutation_payload! { CollateralUpdatePayload, collateral: Collateral }

#[derive(InputObject)]
pub struct CollateralRecordSentToLiquidationInput {
    pub collateral_id: UUID,
    pub amount: Satoshis,
}
crate::mutation_payload! { CollateralRecordSentToLiquidationPayload, collateral: Collateral }

#[derive(InputObject)]
pub struct CollateralRecordProceedsFromLiquidationInput {
    pub collateral_id: UUID,
    pub amount: UsdCents,
}
crate::mutation_payload! { CollateralRecordProceedsFromLiquidationPayload, collateral: Collateral }

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Collateral {
    id: ID,
    collateral_id: UUID,
    pub(crate) wallet_id: Option<UUID>,
    account_id: UUID,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCollateral>,
}

impl From<DomainCollateral> for Collateral {
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
}
