use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::CoreCustodyEvent;
use core_eod::credit_facility_eod::{CREDIT_FACILITY_EOD_JOB_TYPE, CreditFacilityEodConfig};
use core_price::CorePriceEvent;

use super::credit_facility_maturity::{
    CreditFacilityMaturityJobConfig, CreditFacilityMaturityJobSpawner,
};
use super::process_accrual_cycle::{ProcessAccrualCycleJobConfig, ProcessAccrualCycleJobSpawner};
use crate::{CoreCreditEvent, CreditFacilityId, credit_facility::CreditFacilityRepo};

const PAGE_SIZE: i64 = 100;

pub struct CreditFacilityEodJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
    maturity_spawner: CreditFacilityMaturityJobSpawner,
}

impl<E> CreditFacilityEodJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
        maturity_spawner: CreditFacilityMaturityJobSpawner,
    ) -> Self {
        Self {
            credit_facility_repo,
            process_accrual_cycle_spawner,
            maturity_spawner,
        }
    }
}

impl<E> JobInitializer for CreditFacilityEodJobInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = CreditFacilityEodConfig;

    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_EOD_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityEodJobRunner {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            process_accrual_cycle_spawner: self.process_accrual_cycle_spawner.clone(),
            maturity_spawner: self.maturity_spawner.clone(),
        }))
    }
}

struct CreditFacilityEodJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: CreditFacilityEodConfig,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
    maturity_spawner: CreditFacilityMaturityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreditFacilityEodCollectingState {
    accrual_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    maturity_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    accrual_jobs_spawned: usize,
    maturity_jobs_spawned: usize,
    accrual_done: bool,
}

#[async_trait]
impl<E> JobRunner for CreditFacilityEodJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(
        name = "eod.credit-facility-eod.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityEodCollectingState>()?
            .unwrap_or_default();

        // Phase: Collect accrual facilities
        if !state.accrual_done {
            loop {
                let mut op = current_job.begin_op().await?;

                let rows = self
                    .credit_facility_repo
                    .list_facility_ids_eligible_for_accrual_in_op(
                        &mut op,
                        self.config.date,
                        state.accrual_cursor,
                        PAGE_SIZE,
                    )
                    .await?;

                if rows.is_empty() {
                    state.accrual_done = true;
                    current_job
                        .update_execution_state_in_op(&mut op, &state)
                        .await?;
                    op.commit().await?;
                    break;
                }

                let specs: Vec<_> = rows
                    .iter()
                    .map(|(id, _)| {
                        let job_id = core_eod::eod_entity_id(
                            &self.config.date,
                            "interest-accrual",
                            &(*id).into(),
                        );
                        JobSpec::new(
                            job_id,
                            ProcessAccrualCycleJobConfig {
                                credit_facility_id: *id,
                            },
                        )
                        .queue_id(id.to_string())
                    })
                    .collect();

                state.accrual_jobs_spawned += specs.len();
                self.process_accrual_cycle_spawner
                    .spawn_all_in_op(&mut op, specs)
                    .await?;

                state.accrual_cursor = rows.last().map(|(id, ts)| (*ts, *id));
                current_job
                    .update_execution_state_in_op(&mut op, &state)
                    .await?;
                op.commit().await?;
            }
        }

        // Phase: Collect maturing facilities
        loop {
            let rows = self
                .credit_facility_repo
                .list_ids_ready_for_maturity(self.config.date, state.maturity_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    let job_id = core_eod::eod_entity_id(
                        &self.config.date,
                        "credit-maturity",
                        &(*id).into(),
                    );
                    JobSpec::new(
                        job_id,
                        CreditFacilityMaturityJobConfig {
                            credit_facility_id: *id,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            state.maturity_jobs_spawned += specs.len();
            let mut op = current_job.begin_op().await?;
            self.maturity_spawner
                .spawn_all_in_op(&mut op, specs)
                .await?;

            state.maturity_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(&mut op, &state)
                .await?;
            op.commit().await?;
        }

        tracing::info!(
            accrual_jobs = state.accrual_jobs_spawned,
            maturity_jobs = state.maturity_jobs_spawned,
            "Credit facility EOD collection complete"
        );

        Ok(JobCompletion::Complete)
    }
}
