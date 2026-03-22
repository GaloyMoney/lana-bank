use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::*;
use obix::out::OutboxEventMarker;

use super::update_collateralization::{
    UpdateCollateralizationConfig, UpdateCollateralizationSpawner,
};
use crate::{
    CoreCreditEvent, CreditFacilityId, credit_facility::CreditFacilityRepo,
    primitives::PriceOfOneBTC,
};

const COLLECT_FACILITIES_FOR_COLLATERALIZATION_JOB: JobType =
    JobType::new("task.collect-facilities-for-collateralization");
const PAGE_SIZE: i64 = 100;

#[derive(Clone, Serialize, Deserialize)]
pub struct CollectFacilitiesForCollateralizationJobConfig {
    pub price: PriceOfOneBTC,
}

pub struct CollectFacilitiesForCollateralizationJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    credit_facility_repo: CreditFacilityRepo<E>,
    update_collateralization_spawner: UpdateCollateralizationSpawner,
}

impl<E> CollectFacilitiesForCollateralizationJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        credit_facility_repo: &CreditFacilityRepo<E>,
        update_collateralization_spawner: UpdateCollateralizationSpawner,
    ) -> Self {
        Self {
            credit_facility_repo: credit_facility_repo.clone(),
            update_collateralization_spawner,
        }
    }
}

impl<E> JobInitializer for CollectFacilitiesForCollateralizationJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = CollectFacilitiesForCollateralizationJobConfig;

    fn job_type(&self) -> JobType {
        COLLECT_FACILITIES_FOR_COLLATERALIZATION_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CollectFacilitiesForCollateralizationJobRunner {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            update_collateralization_spawner: self.update_collateralization_spawner.clone(),
        }))
    }
}

struct CollectFacilitiesForCollateralizationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CollectFacilitiesForCollateralizationJobConfig,
    credit_facility_repo: CreditFacilityRepo<E>,
    update_collateralization_spawner: UpdateCollateralizationSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct CollectFacilitiesForCollateralizationState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
}

#[async_trait]
impl<E> JobRunner for CollectFacilitiesForCollateralizationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(
        name = "credit.credit_facility.collect_facilities_for_collateralization_job",
        skip(self, current_job),
        fields(price = %self.config.price)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CollectFacilitiesForCollateralizationState>()?
            .unwrap_or_default();

        loop {
            let rows = self
                .credit_facility_repo
                .list_ids_for_price_update(self.config.price, state.last_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let mut op = current_job.begin_op().await?;

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    JobSpec::new(
                        JobId::new(),
                        UpdateCollateralizationConfig {
                            credit_facility_id: *id,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            self.update_collateralization_spawner
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

pub type CollectFacilitiesForCollateralizationJobSpawner =
    JobSpawner<CollectFacilitiesForCollateralizationJobConfig>;
