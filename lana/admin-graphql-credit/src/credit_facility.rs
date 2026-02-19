use async_graphql::*;

use crate::{balance::*, history::*, primitives::*, repayment::*, terms::*};

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
#[graphql(complex)]
pub struct CreditFacilityBase {
    id: ID,
    credit_facility_id: UUID,
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
}

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

#[derive(InputObject)]
pub struct CreditFacilityCompleteInput {
    pub credit_facility_id: UUID,
}

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

#[derive(InputObject)]
pub struct CreditFacilitiesFilter {
    pub status: Option<CreditFacilityStatus>,
    pub collateralization_state: Option<CollateralizationState>,
}
