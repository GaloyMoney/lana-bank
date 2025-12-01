#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod error;
mod event;
mod primitives;
mod proposal;
mod publisher;
mod rbac;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::{Outbox, OutboxEventMarker};

use error::*;
use event::CoreCreditFacilityEvent;
use proposal::CreditFacilityProposalRepo;
use rbac::*;

pub struct CoreCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditFacilityEvent> + OutboxEventMarker<GovernanceEvent>,
{
    authz: Arc<Perms>,
    proposals: CreditFacilityProposalRepo<E>,
}

impl<Perms, E> Clone for CoreCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditFacilityEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            proposals: self.proposals.clone(),
        }
    }
}
impl<Perms, E> CoreCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditFacilityAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditFacilityObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditFacilityEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        outbox: &Outbox<E>,
        authz: Arc<Perms>,
        governance: Arc<Governance<Perms, E>>,
    ) -> Result<Self, CoreCreditFacilityError> {
        let publisher = publisher::CreditFacilityPublisher::new(outbox);
        let proposals = CreditFacilityProposalRepo::new(pool, &publisher);
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
        Ok(Self { authz, proposals })
    }
}
