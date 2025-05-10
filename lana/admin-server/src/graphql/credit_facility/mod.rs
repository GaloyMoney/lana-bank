mod balance;
pub(super) mod disbursal;
mod error;
mod history;
pub(super) mod payment;
mod quote;
mod repayment;

use async_graphql::*;
use quote::CreditFacilityQuoteEntry;

use crate::primitives::*;

use super::{
    approval_process::*, customer::*, loader::LanaDataLoader, primitives::SortDirection, terms::*,
};
pub use lana_app::{
    credit::{
        CreditFacilitiesCursor, CreditFacilitiesSortBy as DomainCreditFacilitiesSortBy,
        CreditFacility as DomainCreditFacility, DisbursalsSortBy as DomainDisbursalsSortBy,
        FacilityCVL, FindManyCreditFacilities, FindManyDisbursals, ListDirection, Sort,
    },
    primitives::CreditFacilityStatus,
};

pub use balance::*;
pub use disbursal::*;
pub use error::*;
pub use history::*;
pub use repayment::*;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacility {
    id: ID,
    credit_facility_id: UUID,
    approval_process_id: UUID,
    activated_at: Option<Timestamp>,
    matures_at: Option<Timestamp>,
    created_at: Timestamp,
    collateralization_state: CollateralizationState,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainCreditFacility>,
}

impl From<DomainCreditFacility> for CreditFacility {
    fn from(credit_facility: DomainCreditFacility) -> Self {
        let activated_at: Option<Timestamp> = credit_facility.activated_at.map(|t| t.into());
        let matures_at: Option<Timestamp> = credit_facility.matures_at.map(|t| t.into());

        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
            approval_process_id: UUID::from(credit_facility.approval_process_id),
            activated_at,
            matures_at,
            created_at: credit_facility.created_at().into(),
            facility_amount: credit_facility.amount,
            collateralization_state: credit_facility.last_collateralization_state(),

            entity: Arc::new(credit_facility),
        }
    }
}

#[ComplexObject]
impl CreditFacility {
    async fn can_be_completed(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().can_be_completed(&self.entity).await?)
    }

    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    async fn status(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityStatus> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .ensure_up_to_date_status(&self.entity)
            .await?
            .map(|cf| cf.status())
            .unwrap_or_else(|| self.entity.status()))
    }

    async fn current_cvl(&self, ctx: &Context<'_>) -> async_graphql::Result<FacilityCVL> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().facility_cvl(&self.entity).await?)
    }

    async fn history(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityHistoryEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().history(sub, self.entity.id).await?)
    }

    async fn repayment_plan(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityRepaymentPlanEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().repayment_plan(sub, self.entity.id).await?)
    }

    async fn quote(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityQuoteEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().quote(sub, self.entity.id).await?)
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityDisbursal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let disbursals = app
            .credit()
            .list_disbursals(
                sub,
                Default::default(),
                FindManyDisbursals::WithCreditFacilityId(self.entity.id),
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

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn subject_can_update_collateral(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_update_collateral(sub, false)
            .await
            .is_ok())
    }

    async fn subject_can_initiate_disbursal(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_initiate_disbursal(sub, false)
            .await
            .is_ok())
    }

    async fn subject_can_record_payment(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .subject_can_record_payment(sub, false)
            .await
            .is_ok())
    }

    async fn subject_can_complete(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
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
        let balance = app.credit().balance(sub, self.entity.id).await?;
        Ok(CreditFacilityBalance::from(balance))
    }
}

#[derive(InputObject)]
pub struct CreditFacilityCreateInput {
    pub customer_id: UUID,
    pub disbursal_credit_account_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
}
crate::mutation_payload! { CreditFacilityCreatePayload, credit_facility: CreditFacility }

#[derive(InputObject)]
pub struct CreditFacilityCollateralUpdateInput {
    pub credit_facility_id: UUID,
    pub collateral: Satoshis,
    pub effective: Date,
}
crate::mutation_payload! { CreditFacilityCollateralUpdatePayload, credit_facility: CreditFacility }

#[derive(InputObject)]
pub struct CreditFacilityPartialPaymentInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
    pub effective: Date,
}
crate::mutation_payload! { CreditFacilityPartialPaymentPayload, credit_facility: CreditFacility }

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
