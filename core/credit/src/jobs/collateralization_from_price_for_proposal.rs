use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::OutboxEventMarker;

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject,
    pending_credit_facility::PendingCreditFacilities,
};

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CreditFacilityProposalCollateralizationFromPriceJobConfig<Perms, E> {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub job_interval: Duration,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for CreditFacilityProposalCollateralizationFromPriceJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    type Initializer = CreditFacilityProposalCollateralizationFromPriceInit<Perms, E>;
}
pub struct CreditFacilityProposalCollateralizationFromPriceInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    credit_facility_proposals: PendingCreditFacilities<Perms, E>,
}

impl<Perms, E> CreditFacilityProposalCollateralizationFromPriceInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(credit_facility_proposals: PendingCreditFacilities<Perms, E>) -> Self {
        Self {
            credit_facility_proposals,
        }
    }
}

const CREDIT_FACILITY_PROPOSAL_COLLATERALZIATION_FROM_PRICE_JOB: JobType =
    JobType::new("credit-facility-proposal-collateralization-from-price");
impl<Perms, E> JobInitializer for CreditFacilityProposalCollateralizationFromPriceInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_PROPOSAL_COLLATERALZIATION_FROM_PRICE_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            CreditFacilityProposalCollateralizationFromPriceJobRunner::<Perms, E> {
                config: job.config()?,
                credit_facility_proposals: self.credit_facility_proposals.clone(),
            },
        ))
    }
}

pub struct CreditFacilityProposalCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    config: CreditFacilityProposalCollateralizationFromPriceJobConfig<Perms, E>,
    credit_facility_proposals: PendingCreditFacilities<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityProposalCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.credit_facility_proposals
            .update_collateralization_from_price()
            .await?;

        Ok(JobCompletion::RescheduleIn(self.config.job_interval))
    }
}
