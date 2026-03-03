use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::*;
use obix::out::OutboxEventMarker;

use super::process_accrual_cycle::{ProcessAccrualCycleJobConfig, ProcessAccrualCycleJobSpawner};
use crate::{CoreCreditEvent, CreditFacilityId, credit_facility::CreditFacilityRepo};

const COLLECT_FACILITIES_FOR_ACCRUAL_JOB: JobType =
    JobType::new("task.collect-facilities-for-accrual");
const PAGE_SIZE: i64 = 100;

#[derive(Clone, Serialize, Deserialize)]
pub struct CollectFacilitiesForAccrualJobConfig {
    pub day: chrono::NaiveDate,
}

pub struct CollectFacilitiesForAccrualJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    credit_facility_repo: CreditFacilityRepo<E>,
    process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
}

impl<E> CollectFacilitiesForAccrualJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        credit_facility_repo: &CreditFacilityRepo<E>,
        process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
    ) -> Self {
        Self {
            credit_facility_repo: credit_facility_repo.clone(),
            process_accrual_cycle_spawner,
        }
    }
}

impl<E> JobInitializer for CollectFacilitiesForAccrualJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = CollectFacilitiesForAccrualJobConfig;

    fn job_type(&self) -> JobType {
        COLLECT_FACILITIES_FOR_ACCRUAL_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CollectFacilitiesForAccrualJobRunner {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            process_accrual_cycle_spawner: self.process_accrual_cycle_spawner.clone(),
        }))
    }
}

struct CollectFacilitiesForAccrualJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CollectFacilitiesForAccrualJobConfig,
    credit_facility_repo: CreditFacilityRepo<E>,
    process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct CollectFacilitiesForAccrualState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
}

#[async_trait]
impl<E> JobRunner for CollectFacilitiesForAccrualJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(
        name = "credit.credit_facility.collect_facilities_for_accrual_job",
        skip(self, current_job),
        fields(day = %self.config.day)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CollectFacilitiesForAccrualState>()?
            .unwrap_or_default();

        loop {
            let mut op = current_job.begin_op().await?;

            let rows = self
                .credit_facility_repo
                .list_facility_ids_eligible_for_accrual_in_op(
                    &mut op,
                    self.config.day,
                    state.last_cursor,
                    PAGE_SIZE,
                )
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    JobSpec::new(
                        JobId::new(),
                        ProcessAccrualCycleJobConfig {
                            credit_facility_id: *id,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            self.process_accrual_cycle_spawner
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

pub type CollectFacilitiesForAccrualJobSpawner = JobSpawner<CollectFacilitiesForAccrualJobConfig>;
