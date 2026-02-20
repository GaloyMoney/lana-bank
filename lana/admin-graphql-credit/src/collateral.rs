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
#[graphql(name = "Collateral", complex)]
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

#[ComplexObject]
impl CollateralBase {
    async fn account(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<admin_graphql_accounting::LedgerAccount> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let accounts: std::collections::HashMap<_, admin_graphql_accounting::LedgerAccount> = app
            .accounting()
            .find_all_ledger_accounts(
                admin_graphql_accounting::CHART_REF,
                &[self.entity.account_ids.collateral_account_id.into()],
            )
            .await?;
        Ok(accounts
            .into_values()
            .next()
            .expect("Collateral account not found"))
    }

    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<crate::credit_facility::CreditFacilityBase>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let facilities: std::collections::HashMap<_, crate::credit_facility::CreditFacilityBase> =
            app.credit()
                .facilities()
                .find_all(&[self.entity.credit_facility_id])
                .await?;
        Ok(facilities.into_values().next())
    }
}
