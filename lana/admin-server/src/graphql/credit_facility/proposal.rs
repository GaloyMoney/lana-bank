use async_graphql::*;

use crate::{
    graphql::{
        custody::Custodian,
        customer::*,
        loader::LanaDataLoader,
        terms::{TermValues, TermsInput},
    },
    primitives::*,
};

use super::{ApprovalProcess, CreditFacilityRepaymentPlanEntry};

pub(crate) use lana_app::credit::{
    CreditFacilityProposal as DomainCreditFacilityProposal,
    CreditFacilityProposalsByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityProposal {
    id: ID,
    credit_facility_proposal_id: UUID,
    approval_process_id: Option<UUID>,
    status: CreditFacilityProposalStatus,
    created_at: Timestamp,
    facility_amount: UsdCents,
    credit_facility_terms: TermValues,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCreditFacilityProposal>,
}

#[ComplexObject]
impl CreditFacilityProposal {
    async fn custodian(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Custodian>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        if let Some(custodian_id) = self.entity.custodian_id {
            let custodian = loader
                .load_one(custodian_id)
                .await?
                .expect("custodian not found");

            return Ok(Some(custodian));
        }
        Ok(None)
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
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

    async fn approval_process(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ApprovalProcess>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        if let Some(approval_process_id) = self.approval_process_id {
            let process = loader
                .load_one(ApprovalProcessId::from(approval_process_id))
                .await?
                .expect("process not found");
            return Ok(Some(process));
        }
        Ok(None)
    }
}

impl From<DomainCreditFacilityProposal> for CreditFacilityProposal {
    fn from(proposal: DomainCreditFacilityProposal) -> Self {
        let created_at = proposal.created_at();

        Self {
            id: proposal.id.to_global_id(),
            credit_facility_proposal_id: UUID::from(proposal.id),
            approval_process_id: proposal.approval_process_id.map(|id| id.into()),
            status: proposal.status(),
            created_at: created_at.into(),
            facility_amount: proposal.amount,
            credit_facility_terms: proposal.terms.into(),

            entity: Arc::new(proposal),
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct CreditFacilityProposalConcludedPayload {
    pub status: CreditFacilityProposalStatus,
    #[graphql(skip)]
    pub credit_facility_proposal_id: CreditFacilityProposalId,
}

#[ComplexObject]
impl CreditFacilityProposalConcludedPayload {
    async fn credit_facility_proposal(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityProposal> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let proposal = loader
            .load_one(self.credit_facility_proposal_id)
            .await?
            .expect("credit facility proposal not found");
        Ok(proposal)
    }
}

#[derive(InputObject)]
pub struct CreditFacilityProposalCreateInput {
    pub customer_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
    pub custodian_id: Option<UUID>,
}
crate::mutation_payload! { CreditFacilityProposalCreatePayload, credit_facility_proposal: CreditFacilityProposal }

#[derive(InputObject)]
pub struct CreditFacilityProposalCustomerApprovalConcludeInput {
    pub credit_facility_proposal_id: UUID,
    pub approved: bool,
}

crate::mutation_payload! { CreditFacilityProposalCustomerApprovalConcludePayload, credit_facility_proposal: CreditFacilityProposal }
