use async_graphql::*;

use crate::{primitives::*, repayment::*, terms::*};

pub use lana_app::credit::{
    CreditFacilityProposal as DomainCreditFacilityProposal,
    CreditFacilityProposalsByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "CreditFacilityProposal", complex)]
pub struct CreditFacilityProposalBase {
    id: ID,
    credit_facility_proposal_id: UUID,
    customer_id: UUID,
    approval_process_id: Option<UUID>,
    status: CreditFacilityProposalStatus,
    created_at: Timestamp,
    facility_amount: UsdCents,
    credit_facility_terms: TermValues,

    #[graphql(skip)]
    pub entity: Arc<DomainCreditFacilityProposal>,
}

#[ComplexObject]
impl CreditFacilityProposalBase {
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

    async fn custodian(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<admin_graphql_custody::Custodian>> {
        if let Some(custodian_id) = self.entity.custodian_id {
            let (app, _sub) = app_and_sub_from_ctx!(ctx);
            let custodians: std::collections::HashMap<_, admin_graphql_custody::Custodian> =
                app.custody().find_all_custodians(&[custodian_id]).await?;
            Ok(custodians.into_values().next())
        } else {
            Ok(None)
        }
    }

    async fn approval_process(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<admin_graphql_governance::ApprovalProcess>> {
        if let Some(approval_process_id) = self.entity.approval_process_id {
            let (app, _sub) = app_and_sub_from_ctx!(ctx);
            let processes: std::collections::HashMap<_, admin_graphql_governance::ApprovalProcess> =
                app.governance()
                    .find_all_approval_processes(&[approval_process_id])
                    .await?;
            Ok(processes.into_values().next())
        } else {
            Ok(None)
        }
    }
}

impl From<DomainCreditFacilityProposal> for CreditFacilityProposalBase {
    fn from(proposal: DomainCreditFacilityProposal) -> Self {
        let created_at = proposal.created_at();

        Self {
            id: proposal.id.to_global_id(),
            credit_facility_proposal_id: UUID::from(proposal.id),
            customer_id: UUID::from(proposal.customer_id),
            approval_process_id: proposal.approval_process_id.map(|id| id.into()),
            status: proposal.status(),
            created_at: created_at.into(),
            facility_amount: proposal.amount,
            credit_facility_terms: proposal.terms.into(),

            entity: Arc::new(proposal),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityProposalCreateInput {
    pub customer_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
    pub custodian_id: Option<UUID>,
}

#[derive(InputObject)]
pub struct CreditFacilityProposalCustomerApprovalConcludeInput {
    pub credit_facility_proposal_id: UUID,
    pub approved: bool,
}
