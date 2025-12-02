#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod collateral;
mod config;
mod error;
mod event;
mod ledger;
mod pending_facility;
mod primitives;
mod proposal;
mod publisher;
mod rbac;

use std::sync::Arc;
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::{Outbox, OutboxEventMarker};

use collateral::*;
use config::*;
use error::*;
use event::CoreCreditFacilityEvent;
use primitives::*;
use proposal::*;
use rbac::*;

pub struct CoreCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditFacilityEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    config: CreditFacilityConfig,
    authz: Arc<Perms>,
    proposals: CreditFacilityProposalRepo<E>,
    collaterals: CollateralRepo<E>,
    customers: Arc<Customers<Perms, E>>,
}

impl<Perms, E> Clone for CoreCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditFacilityEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            authz: self.authz.clone(),
            proposals: self.proposals.clone(),
            customers: self.customers.clone(),
            collaterals: self.collaterals.clone(),
        }
    }
}
impl<Perms, E> CoreCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditFacilityAction> + From<GovernanceAction> + From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditFacilityObject> + From<GovernanceObject> + From<CustomerObject>,
    E: OutboxEventMarker<CoreCreditFacilityEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        config: CreditFacilityConfig,
        outbox: &Outbox<E>,
        authz: Arc<Perms>,
        governance: Arc<Governance<Perms, E>>,
        customers: Arc<Customers<Perms, E>>,
    ) -> Result<Self, CoreCreditFacilityError> {
        let publisher = publisher::CreditFacilityPublisher::new(outbox);
        let proposals = CreditFacilityProposalRepo::new(pool, &publisher);
        let collaterals = CollateralRepo::new(pool, &publisher);
        match governance
            .init_policy(proposal::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS)
            .await
        {
            Err(governance::error::GovernanceError::PolicyError(
                governance::policy_error::PolicyError::DuplicateApprovalProcessType,
            )) => (),
            Err(e) => return Err(e.into()),
            _ => (),
        }
        Ok(Self {
            config,
            authz,
            proposals,
            collaterals,
            customers,
        })
    }

    pub async fn subject_can_create(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CoreCreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditFacilityObject::all_credit_facility_proposals(),
                CoreCreditFacilityAction::CREDIT_FACILITY_PROPOSAL_CREATE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "core_credit_facilities.create_proposal", skip(self),fields(credit_facility_proposal_id = tracing::field::Empty), err)]
    pub async fn create_proposal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug + Copy,
        deposit_account_id: impl Into<CalaAccountId> + std::fmt::Debug,
        amount: UsdCents,
        terms: TermValues,
        custodian_id: Option<impl Into<CustodianId> + std::fmt::Debug + Copy>,
    ) -> Result<CreditFacilityProposal, CoreCreditFacilityError> {
        self.subject_can_create(sub, true)
            .await?
            .expect("audit info missing");

        let customer = self.customers.find_by_id_without_audit(customer_id).await?;
        if self.config.customer_eligibility_check_enabled
            && !customer.eligible_for_credit_facility()
        {
            return Err(CoreCreditFacilityError::CustomerNotEligible);
        }

        let proposal_id = CreditFacilityProposalId::new();
        tracing::Span::current().record(
            "credit_facility_proposal_id",
            tracing::field::display(proposal_id),
        );

        let mut db = self.proposals.begin_op().await?;

        let new_facility_proposal = NewCreditFacilityProposal::builder()
            .id(proposal_id)
            .customer_id(customer.id)
            .customer_type(customer.customer_type)
            .custodian_id(custodian_id.map(|id| id.into()))
            .disbursal_credit_account_id(deposit_account_id)
            .terms(terms)
            .amount(amount)
            .build()
            .expect("could not build new credit facility proposal");

        let credit_facility_proposal = self
            .proposals
            .create_in_op(&mut db, new_facility_proposal)
            .await?;

        db.commit().await?;

        Ok(credit_facility_proposal)
    }
}
