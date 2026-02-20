use async_graphql::*;

use super::{
    balance::*, collateral::DomainCollateral, disbursal::CreditFacilityDisbursalBase, history::*,
    ledger_accounts::CreditFacilityLedgerAccounts, liquidation::LiquidationBase, primitives::*,
    repayment::*, terms::*,
};

pub use lana_app::{
    credit::{
        CreditFacilitiesCursor, CreditFacilitiesFilters as DomainCreditFacilitiesFilters,
        CreditFacilitiesSortBy as DomainCreditFacilitiesSortBy,
        CreditFacility as DomainCreditFacility, DisbursalsFilters,
        DisbursalsSortBy as DomainDisbursalsSortBy, ListDirection, Sort,
    },
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "CreditFacility", complex)]
pub struct CreditFacilityBase {
    id: ID,
    credit_facility_id: UUID,
    customer_id: UUID,
    collateral_id: UUID,
    matures_at: Timestamp,
    activated_at: Timestamp,
    collateralization_state: CollateralizationState,
    status: CreditFacilityStatus,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub entity: Arc<DomainCreditFacility>,
}

impl From<DomainCreditFacility> for CreditFacilityBase {
    fn from(credit_facility: DomainCreditFacility) -> Self {
        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
            customer_id: UUID::from(credit_facility.customer_id),
            collateral_id: UUID::from(credit_facility.collateral_id),
            activated_at: Timestamp::from(credit_facility.activated_at),
            matures_at: Timestamp::from(credit_facility.matures_at()),
            facility_amount: credit_facility.amount,
            status: credit_facility.status(),
            collateralization_state: credit_facility.last_collateralization_state(),

            entity: Arc::new(credit_facility),
        }
    }
}

#[ComplexObject]
impl CreditFacilityBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn can_be_completed(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, _) = app_and_sub_from_ctx!(ctx);
        Ok(app.credit().can_be_completed(&self.entity).await?)
    }

    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    async fn current_cvl(&self, ctx: &Context<'_>) -> async_graphql::Result<CVLPct> {
        let (app, _) = app_and_sub_from_ctx!(ctx);
        Ok(app.credit().current_cvl(&self.entity).await?.into())
    }

    async fn history(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityHistoryEntry>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        Ok(app
            .credit()
            .histories()
            .find_for_credit_facility_id(sub, self.entity.id)
            .await?)
    }

    async fn repayment_plan(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityRepaymentPlanEntry>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .repayment_plans()
            .find_for_credit_facility_id(sub, self.entity.id)
            .await?)
    }

    async fn user_can_update_collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .collaterals()
            .subject_can_update_collateral(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_initiate_disbursal(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_initiate_disbursal(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_record_payment(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_record_payment(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_record_payment_with_date(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_record_payment_with_date(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_complete(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app.credit().subject_can_complete(sub, false).await.is_ok())
    }

    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityBalance> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let balance = app
            .credit()
            .facilities()
            .balance(sub, self.entity.id)
            .await?;
        Ok(CreditFacilityBalance::from(balance))
    }

    async fn wallet(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<crate::custody::WalletBase>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let collaterals: std::collections::HashMap<_, DomainCollateral> = app
            .credit()
            .collaterals()
            .find_all(&[self.entity.collateral_id])
            .await?;
        let collateral = collaterals
            .into_values()
            .next()
            .expect("credit facility has collateral");

        if let Some(wallet_id) = collateral.custody_wallet_id {
            let wallets: std::collections::HashMap<_, crate::custody::WalletBase> =
                app.custody().find_all_wallets(&[wallet_id]).await?;
            Ok(wallets.into_values().next())
        } else {
            Ok(None)
        }
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityDisbursalBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let disbursals = app
            .credit()
            .disbursals()
            .list(
                sub,
                Default::default(),
                DisbursalsFilters {
                    credit_facility_id: Some(self.entity.id),
                    ..Default::default()
                },
                Sort {
                    by: DomainDisbursalsSortBy::CreatedAt,
                    direction: ListDirection::Descending,
                },
            )
            .await?;

        Ok(disbursals
            .entities
            .into_iter()
            .map(CreditFacilityDisbursalBase::from)
            .collect())
    }

    async fn liquidations(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<LiquidationBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let liquidations = app
            .credit()
            .collaterals()
            .list_liquidations_for_collateral_by_created_at(
                sub,
                self.entity.collateral_id,
                Default::default(),
            )
            .await?;

        Ok(liquidations
            .entities
            .into_iter()
            .map(LiquidationBase::from)
            .collect())
    }

    async fn ledger_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityLedgerAccounts> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let collaterals: std::collections::HashMap<_, DomainCollateral> = app
            .credit()
            .collaterals()
            .find_all(&[self.entity.collateral_id])
            .await?;
        let collateral = collaterals
            .into_values()
            .next()
            .expect("credit facility has collateral");

        Ok(CreditFacilityLedgerAccounts {
            facility_account_id: self.entity.account_ids.facility_account_id.into(),
            disbursed_receivable_not_yet_due_account_id: self
                .entity
                .account_ids
                .disbursed_receivable_not_yet_due_account_id
                .into(),
            disbursed_receivable_due_account_id: self
                .entity
                .account_ids
                .disbursed_receivable_due_account_id
                .into(),
            disbursed_receivable_overdue_account_id: self
                .entity
                .account_ids
                .disbursed_receivable_overdue_account_id
                .into(),
            disbursed_defaulted_account_id: self
                .entity
                .account_ids
                .disbursed_defaulted_account_id
                .into(),
            collateral_account_id: collateral.account_ids.collateral_account_id.into(),
            collateral_in_liquidation_account_id: collateral
                .account_ids
                .collateral_in_liquidation_account_id
                .into(),
            liquidated_collateral_account_id: collateral
                .account_ids
                .liquidated_collateral_account_id
                .into(),
            proceeds_from_liquidation_account_id: self
                .entity
                .account_ids
                .proceeds_from_liquidation_account_id
                .into_inner()
                .into(),
            interest_receivable_not_yet_due_account_id: self
                .entity
                .account_ids
                .interest_receivable_not_yet_due_account_id
                .into(),
            interest_receivable_due_account_id: self
                .entity
                .account_ids
                .interest_receivable_due_account_id
                .into(),
            interest_receivable_overdue_account_id: self
                .entity
                .account_ids
                .interest_receivable_overdue_account_id
                .into(),
            interest_defaulted_account_id: self
                .entity
                .account_ids
                .interest_defaulted_account_id
                .into(),
            interest_income_account_id: self.entity.account_ids.interest_income_account_id.into(),
            fee_income_account_id: self.entity.account_ids.fee_income_account_id.into(),
            payment_holding_account_id: self.entity.account_ids.payment_holding_account_id.into(),
            uncovered_outstanding_account_id: self
                .entity
                .account_ids
                .uncovered_outstanding_account_id
                .into(),
        })
    }
}
