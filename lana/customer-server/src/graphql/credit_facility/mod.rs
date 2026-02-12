mod balance;
pub(super) mod disbursal;
mod history;
pub(super) mod payment_allocation;
mod repayment;

use async_graphql::*;

pub(super) use lana_app::credit::{
    CreditFacility as DomainCreditFacility, DisbursalsSortBy as DomainDisbursalsSortBy,
    ListDirection, Sort,
};

use crate::{LanaApp, primitives::*};

use super::terms::*;

use balance::*;
use disbursal::*;
use history::*;
use repayment::*;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(super) struct CreditFacility {
    id: ID,
    credit_facility_id: UUID,
    facility_amount: UsdCents,
    collateralization_state: CollateralizationState,
    status: CreditFacilityStatus,
    activated_at: Timestamp,
    matures_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainCreditFacility>,
}

impl From<DomainCreditFacility> for CreditFacility {
    fn from(credit_facility: DomainCreditFacility) -> Self {
        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
            matures_at: Timestamp::from(credit_facility.matures_at()),
            activated_at: Timestamp::from(credit_facility.activated_at),
            facility_amount: credit_facility.amount,
            collateralization_state: credit_facility.last_collateralization_state(),
            status: credit_facility.status(),

            entity: Arc::new(credit_facility),
        }
    }
}

#[ComplexObject]
impl CreditFacility {
    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityBalance> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let balance = app
            .credit()
            .for_subject(sub)?
            .balance(self.entity.id)
            .await?;

        Ok(CreditFacilityBalance::from(balance))
    }

    async fn current_cvl(&self, ctx: &Context<'_>) -> async_graphql::Result<CVLPct> {
        let app = ctx.data_unchecked::<LanaApp>();
        Ok(app.credit().current_cvl(&self.entity).await?)
    }

    async fn history(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityHistoryEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .for_subject(sub)?
            .history(self.entity.id)
            .await?)
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityDisbursal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let disbursals = app
            .credit()
            .for_subject(sub)?
            .list_disbursals_for_credit_facility(
                self.entity.id,
                Default::default(),
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

    async fn repayment_plan(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<CreditFacilityRepaymentPlanEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .for_subject(sub)?
            .repayment_plan(self.entity.id)
            .await?)
    }
}
