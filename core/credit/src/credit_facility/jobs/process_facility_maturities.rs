use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use std::sync::Arc;

use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::CoreCustodyEvent;
use core_price::CorePriceEvent;
use core_time_events::CoreTimeEvent;

use super::credit_facility_maturity::{
    CreditFacilityMaturityJobConfig, CreditFacilityMaturityJobSpawner,
};
use crate::{CoreCreditEvent, credit_facility::CreditFacilityRepo, primitives::*};

const PROCESS_FACILITY_MATURITIES_JOB: JobType = JobType::new("task.process-facility-maturities");
const PAGE_SIZE: i64 = 100;

#[derive(Serialize, Deserialize)]
pub struct ProcessFacilityMaturitiesJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub day: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for ProcessFacilityMaturitiesJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            day: self.day,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct ProcessFacilityMaturitiesJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    repo: Arc<CreditFacilityRepo<E>>,
    maturity_spawner: CreditFacilityMaturityJobSpawner<E>,
}

impl<E> ProcessFacilityMaturitiesJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        repo: Arc<CreditFacilityRepo<E>>,
        maturity_spawner: CreditFacilityMaturityJobSpawner<E>,
    ) -> Self {
        Self {
            repo,
            maturity_spawner,
        }
    }
}

impl<E> JobInitializer for ProcessFacilityMaturitiesJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = ProcessFacilityMaturitiesJobConfig<E>;

    fn job_type(&self) -> JobType {
        PROCESS_FACILITY_MATURITIES_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ProcessFacilityMaturitiesJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            maturity_spawner: self.maturity_spawner.clone(),
        }))
    }
}

pub struct ProcessFacilityMaturitiesJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: ProcessFacilityMaturitiesJobConfig<E>,
    repo: Arc<CreditFacilityRepo<E>>,
    maturity_spawner: CreditFacilityMaturityJobSpawner<E>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct ProcessFacilityMaturitiesState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
}

#[async_trait]
impl<E> JobRunner for ProcessFacilityMaturitiesJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(
        name = "credit.credit_facility.process_facility_maturities_job",
        skip(self, current_job),
        fields(day = %self.config.day)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<ProcessFacilityMaturitiesState>()?
            .unwrap_or_default();

        loop {
            let rows = self
                .repo
                .list_ids_ready_for_maturity(self.config.day, state.last_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    JobSpec::new(
                        JobId::new(),
                        CreditFacilityMaturityJobConfig::<E> {
                            credit_facility_id: *id,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            let mut op = current_job.begin_op().await?;
            self.maturity_spawner
                .spawn_all_in_op(&mut op, specs)
                .await?;

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(&mut op, &state)
                .await?;
            op.commit().await?;
        }

        Ok(JobCompletion::Complete)
    }
}

pub type ProcessFacilityMaturitiesJobSpawner<E> = JobSpawner<ProcessFacilityMaturitiesJobConfig<E>>;
