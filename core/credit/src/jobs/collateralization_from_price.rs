use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilities, primitives::*, CoreCreditAction, CoreCreditEvent,
    CoreCreditObject,
};

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityCollateralizationFromPriceJobConfig<Perms, E> {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub job_interval: Duration,
    pub upgrade_buffer_cvl_pct: CVLPct,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for CreditFacilityCollateralizationFromPriceJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>;
}
pub struct CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    credit_facilities: CreditFacilities<Perms, E>,
}

impl<Perms, E> CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(credit_facilities: CreditFacilities<Perms, E>) -> Self {
        Self { credit_facilities }
    }
}

const CREDIT_FACILITY_COLLATERALZIATION_FROM_PRICE_JOB: JobType =
    JobType::new("credit-facility-collateralization-from-price");
impl<Perms, E> JobInitializer for CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
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
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    config: CreditFacilityCollateralizationFromPriceJobConfig<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.credit_facilities
            .update_collateralization_from_price(self.config.upgrade_buffer_cvl_pct)
            .await?;

        Ok(JobCompletion::RescheduleIn(self.config.job_interval))
    }
}
