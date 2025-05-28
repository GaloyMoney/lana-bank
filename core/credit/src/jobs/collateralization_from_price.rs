use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilities, ledger::CreditLedger, primitives::*, CoreCreditAction,
    CoreCreditEvent, CoreCreditObject,
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
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>;
}
pub struct CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    credit_facilities: CreditFacilities<Perms, E>,
    ledger: CreditLedger,
    audit: Perms::Audit,
    price: Price,
}

impl<Perms, E> CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        credit_facilities: CreditFacilities<Perms, E>,
        ledger: &CreditLedger,
        price: &Price,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            credit_facilities,
            ledger: ledger.clone(),
            price: price.clone(),
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_COLLATERALZIATION_FROM_PRICE_JOB: JobType =
    JobType::new("credit-facility-collateralization-from-price");
impl<Perms, E> JobInitializer for CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
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
                ledger: self.ledger.clone(),
                price: self.price.clone(),
                audit: self.audit.clone(),
            },
        ))
    }
}

pub struct CreditFacilityCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityCollateralizationFromPriceJobConfig<Perms, E>,
    ledger: CreditLedger,
    credit_facilities: CreditFacilities<Perms, E>,
    price: Price,
    audit: Perms::Audit,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
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
