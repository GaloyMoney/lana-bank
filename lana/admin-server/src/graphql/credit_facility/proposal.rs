use async_graphql::*;

use crate::{
    graphql::{
        custody::Wallet,
        customer::*,
        loader::LanaDataLoader,
        terms::{TermValues, TermsInput},
    },
    primitives::*,
};

use super::{ApprovalProcess, CollateralBalance, CreditFacilityRepaymentPlanEntry};

pub use lana_app::credit::CreditFacilityProposal as DomainCreditFacilityProposal;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityProposal {
    id: ID,
    credit_facility_proposal_id: UUID,
    approval_process_id: UUID,
    created_at: Timestamp,
    // collateralization_state: CreditFacilityProposalCollateralizationState,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCreditFacilityProposal>,
}

#[ComplexObject]
impl CreditFacilityProposal {
    // async fn wallet(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Wallet>> {
    //     let loader = ctx.data_unchecked::<LanaDataLoader>();
    //     let collateral = loader
    //         .load_one(self.entity.collateral_id)
    //         .await?
    //         .expect("credit facility proposal has collateral");

    //     if let Some(wallet_id) = collateral.wallet_id {
    //         Ok(loader.load_one(WalletId::from(wallet_id)).await?)
    //     } else {
    //         Ok(None)
    //     }
    // }

    // async fn collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<CollateralBalance> {
    //     let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

    //     let collateral = app
    //         .credit()
    //         .pending_credit_facilities()
    //         .collateral(sub, self.entity.id)
    //         .await?;

    //     Ok(CollateralBalance {
    //         btc_balance: collateral,
    //     })
    // }

    // async fn status(
    //     &self,
    //     ctx: &Context<'_>,
    // ) -> async_graphql::Result<CreditFacilityProposalStatus> {
    //     let (app, _) = crate::app_and_sub_from_ctx!(ctx);
    //     Ok(app
    //         .credit()
    //         .ensure_up_to_date_proposal_status(&self.entity)
    //         .await?
    //         .map(|cf| cf.status())
    //         .unwrap_or_else(|| self.entity.status()))
    // }

    // async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
    //     let loader = ctx.data_unchecked::<LanaDataLoader>();
    //     let customer = loader
    //         .load_one(self.entity.customer_id)
    //         .await?
    //         .expect("customer not found");
    //     Ok(customer)
    // }

    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    // async fn repayment_plan(
    //     &self,
    //     ctx: &Context<'_>,
    // ) -> async_graphql::Result<Vec<CreditFacilityRepaymentPlanEntry>> {
    //     let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
    //     Ok(app.credit().repayment_plan(sub, self.entity.id).await?)
    // }

    // async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
    //     let loader = ctx.data_unchecked::<LanaDataLoader>();
    //     let process = loader
    //         .load_one(self.entity.approval_process_id)
    //         .await?
    //         .expect("process not found");
    //     Ok(process)
    // }
}

impl From<DomainCreditFacilityProposal> for CreditFacilityProposal {
    fn from(proposal: DomainCreditFacilityProposal) -> Self {
        let created_at = proposal.created_at();

        Self {
            id: proposal.id.to_global_id(),
            credit_facility_proposal_id: UUID::from(proposal.id),
            approval_process_id: UUID::from(proposal.approval_process_id),
            created_at: created_at.into(),
            facility_amount: proposal.amount,
            // collateralization_state: pending_credit_facility.last_collateralization_state(),
            entity: Arc::new(proposal),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityProposalCreateInput {
    pub customer_id: UUID,
    pub disbursal_credit_account_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
    pub custodian_id: Option<UUID>,
}
crate::mutation_payload! { CreditFacilityProposalCreatePayload, credit_facility_proposal: CreditFacilityProposal }
