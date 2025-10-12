use async_graphql::*;

use crate::{
    graphql::{custody::Wallet, customer::*, loader::LanaDataLoader, terms::TermValues},
    primitives::*,
};

use super::{ApprovalProcess, CollateralBalance, CreditFacilityRepaymentPlanEntry};

pub use lana_app::credit::{
    PendingCreditFacilitiesByCreatedAtCursor, PendingCreditFacility as DomainPendingCreditFacility,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PendingCreditFacility {
    id: ID,
    credit_facility_proposal_id: UUID,
    approval_process_id: UUID,
    created_at: Timestamp,
    collateralization_state: PendingCreditFacilityCollateralizationState,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainPendingCreditFacility>,
}

#[ComplexObject]
impl PendingCreditFacility {
    async fn wallet(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Wallet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("credit facility proposal has collateral");

        if let Some(wallet_id) = collateral.wallet_id {
            Ok(loader.load_one(WalletId::from(wallet_id)).await?)
        } else {
            Ok(None)
        }
    }

    async fn collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<CollateralBalance> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let collateral = app
            .credit()
            .pending_credit_facilities()
            .collateral(sub, self.entity.id)
            .await?;

        Ok(CollateralBalance {
            btc_balance: collateral,
        })
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    async fn repayment_plan(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityRepaymentPlanEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().repayment_plan(sub, self.entity.id).await?)
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }
}

impl From<DomainPendingCreditFacility> for PendingCreditFacility {
    fn from(pending_credit_facility: DomainPendingCreditFacility) -> Self {
        let created_at = pending_credit_facility.created_at();

        Self {
            id: pending_credit_facility.id.to_global_id(),
            credit_facility_proposal_id: UUID::from(pending_credit_facility.id),
            approval_process_id: UUID::from(pending_credit_facility.approval_process_id),
            created_at: created_at.into(),
            facility_amount: pending_credit_facility.amount,
            collateralization_state: pending_credit_facility.last_collateralization_state(),

            entity: Arc::new(pending_credit_facility),
        }
    }
}

#[derive(InputObject)]
pub struct PendingCreditFacilityCollateralUpdateInput {
    pub pending_credit_facility_id: UUID,
    pub collateral: Satoshis,
    pub effective: Date,
}
crate::mutation_payload! { PendingCreditFacilityCollateralUpdatePayload, pending_credit_facility: PendingCreditFacility }
