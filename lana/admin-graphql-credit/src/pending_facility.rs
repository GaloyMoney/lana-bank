use async_graphql::*;

use crate::{balance::CollateralBalance, primitives::*, repayment::*, terms::*};

pub use lana_app::credit::{
    PendingCreditFacilitiesByCreatedAtCursor, PendingCreditFacility as DomainPendingCreditFacility,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PendingCreditFacilityBase {
    id: ID,
    pending_credit_facility_id: UUID,
    collateral_id: UUID,
    status: PendingCreditFacilityStatus,
    approval_process_id: UUID,
    created_at: Timestamp,
    collateralization_state: PendingCreditFacilityCollateralizationState,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub entity: Arc<DomainPendingCreditFacility>,
}

#[ComplexObject]
impl PendingCreditFacilityBase {
    async fn collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<CollateralBalance> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let collateral = app
            .credit()
            .pending_credit_facilities()
            .collateral(sub, self.entity.id)
            .await?;

        Ok(CollateralBalance {
            btc_balance: collateral,
        })
    }

    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
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
}

impl From<DomainPendingCreditFacility> for PendingCreditFacilityBase {
    fn from(pending_credit_facility: DomainPendingCreditFacility) -> Self {
        let created_at = pending_credit_facility.created_at();

        Self {
            id: pending_credit_facility.id.to_global_id(),
            pending_credit_facility_id: UUID::from(pending_credit_facility.id),
            collateral_id: UUID::from(pending_credit_facility.collateral_id),
            approval_process_id: UUID::from(pending_credit_facility.approval_process_id),
            created_at: created_at.into(),
            facility_amount: pending_credit_facility.amount,
            collateralization_state: pending_credit_facility.last_collateralization_state(),
            status: pending_credit_facility.status(),

            entity: Arc::new(pending_credit_facility),
        }
    }
}
