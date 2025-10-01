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
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, credit_facility::CreditFacilities,
    primitives::*,
};

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CreditFacilityCollateralizationFromPriceJobConfig<Perms, E> {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub job_interval: Duration,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for CreditFacilityCollateralizationFromPriceJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Initializer = CreditFacilityCollateralizationFromPriceInit<Perms, E>;
}
pub struct CreditFacilityCollateralizationFromPriceInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    credit_facilities: CreditFacilities<Perms, E>,
}

impl<Perms, E> CreditFacilityCollateralizationFromPriceInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(credit_facilities: CreditFacilities<Perms, E>) -> Self {
        Self { credit_facilities }
    }
}

const CREDIT_FACILITY_COLLATERALZIATION_FROM_PRICE_JOB: JobType =
    JobType::new("credit-facility-collateralization-from-price");
impl<Perms, E> JobInitializer for CreditFacilityCollateralizationFromPriceInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_COLLATERALZIATION_FROM_PRICE_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            CreditFacilityCollateralizationFromPriceJobRunner::<Perms, E> {
                config: job.config()?,
                credit_facilities: self.credit_facilities.clone(),
            },
        ))
    }
}

pub struct CreditFacilityCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: CreditFacilityCollateralizationFromPriceJobConfig<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.credit_facilities
            .update_collateralization_from_price(CVLPct::UPGRADE_BUFFER)
            .await?;

        Ok(JobCompletion::RescheduleIn(self.config.job_interval))
    }
}
