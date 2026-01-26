mod balance;
mod collateral;
pub(super) mod disbursal;
mod error;
mod history;
mod ledger_accounts;
mod liquidation;
pub(super) mod payment_allocation;
mod pending_facility;
mod proposal;
mod repayment;

use async_graphql::*;

use crate::primitives::*;

use super::{
    approval_process::ApprovalProcess, custody::Wallet, customer::*, loader::LanaDataLoader,
    primitives::SortDirection, terms::*,
};
pub use lana_app::{
    credit::{
        CreditFacilitiesCursor, CreditFacilitiesFilter as DomainCreditFacilitiesFilter,
        CreditFacilitiesSortBy as DomainCreditFacilitiesSortBy,
        CreditFacility as DomainCreditFacility, DisbursalsFilter,
        DisbursalsSortBy as DomainDisbursalsSortBy, ListDirection, Sort,
    },
    custody::WalletId,
    primitives::CreditFacilityStatus,
    public_id::PublicId,
};

pub use balance::*;
pub use collateral::*;
pub use disbursal::*;
pub use error::*;
pub use history::*;
use ledger_accounts::*;
pub use liquidation::*;
pub use pending_facility::*;
pub use proposal::*;
pub use repayment::*;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacility {
    id: ID,
    credit_facility_id: UUID,
    matures_at: Timestamp,
    activated_at: Timestamp,
    collateralization_state: CollateralizationState,
    status: CreditFacilityStatus,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainCreditFacility>,
}

impl From<DomainCreditFacility> for CreditFacility {
    fn from(credit_facility: DomainCreditFacility) -> Self {
        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
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
impl CreditFacility {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }
    async fn can_be_completed(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().can_be_completed(&self.entity).await?)
    }

    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    async fn current_cvl(&self, ctx: &Context<'_>) -> async_graphql::Result<CVLPct> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().current_cvl(&self.entity).await?.into())
    }

    async fn history(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityHistoryEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

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
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .repayment_plans()
            .find_for_credit_facility_id(sub, self.entity.id)
            .await?)
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityDisbursal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let disbursals = app
            .credit()
            .disbursals()
            .list(
                sub,
                Default::default(),
                DisbursalsFilter::WithCreditFacilityId(self.entity.id),
                Sort {
                    by: DomainDisbursalsSortBy::CreatedAt,
                    direction: ListDirection::Descending,
                },
            )
            .await?;

        Ok(disbursals
            .entities
            .into_iter()
            .map(CreditFacilityDisbursal::from)
            .collect())
    }

    async fn liquidations(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Liquidation>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let liquidations = app
            .credit()
            .liquidations()
            .list_for_facility_by_created_at(sub, self.entity.id)
            .await?
            .into_iter()
            .map(Liquidation::from)
            .collect();

        Ok(liquidations)
    }

    async fn user_can_update_collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_update_collateral(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_initiate_disbursal(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_initiate_disbursal(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_record_payment(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
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
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_record_payment_with_date(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_complete(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().subject_can_complete(sub, false).await.is_ok())
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityBalance> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let balance = app
            .credit()
            .facilities()
            .balance(sub, self.entity.id)
            .await?;
        Ok(CreditFacilityBalance::from(balance))
    }

    async fn wallet(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Wallet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("credit facility has collateral");

        if let Some(wallet_id) = collateral.wallet_id {
            Ok(loader.load_one(WalletId::from(wallet_id)).await?)
        } else {
            Ok(None)
        }
    }

    async fn ledger_accounts(&self) -> CreditFacilityLedgerAccounts {
        CreditFacilityLedgerAccounts {
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
            collateral_account_id: self.entity.account_ids.collateral_account_id.into(),
            // collateral_in_liquidation_account_id: self
                // .entity
                // .account_ids
                // .collateral_in_liquidation_account_id
                // .into(),
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
        }
    }
}

#[derive(InputObject)]
pub struct CollateralUpdateInput {
    pub collateral_id: UUID,
    pub collateral: Satoshis,
    pub effective: Date,
}
crate::mutation_payload! { CollateralUpdatePayload, collateral: Collateral }

#[derive(InputObject)]
pub struct CreditFacilityPartialPaymentRecordInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}

#[derive(InputObject)]
pub struct CreditFacilityPartialPaymentWithDateRecordInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
    pub effective: Date,
}
crate::mutation_payload! { CreditFacilityPartialPaymentRecordPayload, credit_facility: CreditFacility }

#[derive(InputObject)]
pub struct CreditFacilityCompleteInput {
    pub credit_facility_id: UUID,
}
crate::mutation_payload! { CreditFacilityCompletePayload, credit_facility: CreditFacility }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreditFacilitiesSortBy {
    #[default]
    CreatedAt,
    Cvl,
}

impl From<CreditFacilitiesSortBy> for DomainCreditFacilitiesSortBy {
    fn from(by: CreditFacilitiesSortBy) -> Self {
        match by {
            CreditFacilitiesSortBy::CreatedAt => DomainCreditFacilitiesSortBy::CreatedAt,
            CreditFacilitiesSortBy::Cvl => DomainCreditFacilitiesSortBy::CollateralizationRatio,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct CreditFacilitiesSort {
    #[graphql(default)]
    pub by: CreditFacilitiesSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<CreditFacilitiesSort> for Sort<DomainCreditFacilitiesSortBy> {
    fn from(sort: CreditFacilitiesSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<CreditFacilitiesSort> for DomainCreditFacilitiesSortBy {
    fn from(sort: CreditFacilitiesSort) -> Self {
        sort.by.into()
    }
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditFacilitiesFilterBy {
    Status,
    CollateralizationState,
}

#[derive(InputObject)]
pub struct CreditFacilitiesFilter {
    pub field: CreditFacilitiesFilterBy,
    pub status: Option<CreditFacilityStatus>,
    pub collateralization_state: Option<CollateralizationState>,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityCollateralizationPayload {
    #[graphql(flatten)]
    pub update: CreditFacilityCollateralizationUpdated,
    #[graphql(skip)]
    pub credit_facility_id: CreditFacilityId,
}

#[ComplexObject]
impl CreditFacilityCollateralizationPayload {
    async fn credit_facility(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.credit_facility_id)
            .await?
            .expect("credit facility not found");
        Ok(facility)
    }
}
