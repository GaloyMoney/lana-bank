use async_graphql::*;

use super::{
    balance::CollateralBalance, collateral::DomainCollateral, primitives::*, repayment::*, terms::*,
};

pub use lana_app::credit::{
    PendingCreditFacilitiesByCreatedAtCursor, PendingCreditFacility as DomainPendingCreditFacility,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "PendingCreditFacility", complex)]
pub struct PendingCreditFacilityBase {
    id: ID,
    pending_credit_facility_id: UUID,
    customer_id: UUID,
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
            .expect("pending credit facility has collateral");

        if let Some(wallet_id) = collateral.custody_wallet_id {
            let wallets: std::collections::HashMap<_, crate::custody::WalletBase> =
                app.custody().find_all_wallets(&[wallet_id]).await?;
            Ok(wallets.into_values().next())
        } else {
            Ok(None)
        }
    }

    async fn approval_process(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<crate::governance::ApprovalProcess> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let processes: std::collections::HashMap<_, crate::governance::ApprovalProcess> = app
            .governance()
            .find_all_approval_processes(&[self.entity.approval_process_id])
            .await?;
        Ok(processes
            .into_values()
            .next()
            .expect("pending credit facility must have an approval process"))
    }
}

impl From<DomainPendingCreditFacility> for PendingCreditFacilityBase {
    fn from(pending_credit_facility: DomainPendingCreditFacility) -> Self {
        let created_at = pending_credit_facility.created_at();

        Self {
            id: pending_credit_facility.id.to_global_id(),
            pending_credit_facility_id: UUID::from(pending_credit_facility.id),
            customer_id: UUID::from(pending_credit_facility.customer_id),
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
