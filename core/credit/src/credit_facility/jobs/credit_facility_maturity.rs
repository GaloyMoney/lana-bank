use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use std::sync::Arc;

use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::CoreCustodyEvent;
use core_price::CorePriceEvent;

use crate::{CoreCreditEvent, credit_facility::CreditFacilityRepo, primitives::*};

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityMaturityJobConfig<E> {
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

pub struct CreditFacilityMaturityInit<E>
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
    pub fn new(credit_facility_repo: Arc<CreditFacilityRepo<E>>) -> Self {
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

pub struct CreditFacilityMaturityJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: CreditFacilityMaturityJobConfig<E>,
    repo: Arc<CreditFacilityRepo<E>>,
}

#[async_trait]
impl<E> JobRunner for CreditFacilityMaturityJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(
        name = "credit.credit_facility.mark_as_matured",
        skip(self, current_job),
        fields(credit_facility_id = %self.config.credit_facility_id)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        let mut facility = self
            .repo
            .find_by_id_in_op(&mut op, self.config.credit_facility_id)
            .await?;

        if facility.mature().did_execute() {
            self.repo.update_in_op(&mut op, &mut facility).await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub type CreditFacilityMaturityJobSpawner<E> = JobSpawner<CreditFacilityMaturityJobConfig<E>>;
