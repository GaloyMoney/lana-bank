use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use std::sync::Arc;

use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::CoreCustodyEvent;
use core_price::CorePriceEvent;

use crate::{
    credit_facility::{CreditFacilityError, CreditFacilityRepo},
    event::CoreCreditEvent,
    primitives::*,
};

#[derive(Serialize, Deserialize)]
pub(crate) struct CreditFacilityMaturityJobConfig<E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for CreditFacilityMaturityJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            credit_facility_id: self.credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) struct CreditFacilityMaturityInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    repo: Arc<CreditFacilityRepo<E>>,
}

impl<E> CreditFacilityMaturityInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub(crate) fn new(credit_facility_repo: Arc<CreditFacilityRepo<E>>) -> Self {
        Self {
            repo: credit_facility_repo,
        }
    }
}

const CREDIT_FACILITY_MATURITY_JOB: JobType = JobType::new("task.credit-facility-maturity");
impl<E> JobInitializer for CreditFacilityMaturityInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = CreditFacilityMaturityJobConfig<E>;
    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_MATURITY_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityMaturityJobRunner::<E> {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

pub(crate) struct CreditFacilityMaturityJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: CreditFacilityMaturityJobConfig<E>,
    repo: Arc<CreditFacilityRepo<E>>,
}

impl<E> CreditFacilityMaturityJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.mark_as_matured",
        skip(self),
        fields(credit_facility_id = %credit_facility_id)
    )]
    async fn mark_facility_as_matured(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<(), CreditFacilityError> {
        let mut facility = self.repo.find_by_id(credit_facility_id).await?;

        if facility.mature().did_execute() {
            self.repo.update(&mut facility).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for CreditFacilityMaturityJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.mark_facility_as_matured(self.config.credit_facility_id)
            .await?;
        Ok(JobCompletion::Complete)
    }
}

pub(crate) type CreditFacilityMaturityJobSpawner<E> =
    JobSpawner<CreditFacilityMaturityJobConfig<E>>;
