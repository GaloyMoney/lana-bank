use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use governance::GovernanceEvent;
use job::{error::JobError, *};
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
    jobs: Jobs,
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
        jobs: &Jobs,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
        maturity_spawner: CreditFacilityMaturityJobSpawner,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
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
            jobs: self.jobs.clone(),
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
    jobs: Jobs,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
    maturity_spawner: CreditFacilityMaturityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CreditFacilityEodState {
    #[default]
    Collecting(CreditFacilityEodCollectingState),
    Tracking {
        accrual_job_ids: Vec<JobId>,
        maturity_job_ids: Vec<JobId>,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreditFacilityEodCollectingState {
    accrual_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    maturity_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    accrual_job_ids: Vec<JobId>,
    maturity_job_ids: Vec<JobId>,
    accrual_done: bool,
}

impl<E> CreditFacilityEodJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    async fn run_collecting(
        &self,
        mut current_job: CurrentJob,
        mut state: CreditFacilityEodCollectingState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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
                        .update_execution_state_in_op(
                            &mut op,
                            &CreditFacilityEodState::Collecting(state.clone()),
                        )
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
                        state.accrual_job_ids.push(job_id);
                        JobSpec::new(
                            job_id,
                            ProcessAccrualCycleJobConfig {
                                credit_facility_id: *id,
                            },
                        )
                        .queue_id(id.to_string())
                    })
                    .collect();

                match self
                    .process_accrual_cycle_spawner
                    .spawn_all_in_op(&mut op, specs)
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                state.accrual_cursor = rows.last().map(|(id, ts)| (*ts, *id));
                current_job
                    .update_execution_state_in_op(
                        &mut op,
                        &CreditFacilityEodState::Collecting(state.clone()),
                    )
                    .await?;
                op.commit().await?;
            }
        }

        // Phase: Collect maturing facilities
        loop {
            let mut op = current_job.begin_op().await?;

            let rows = self
                .credit_facility_repo
                .list_ids_ready_for_maturity_in_op(
                    &mut op,
                    self.config.date,
                    state.maturity_cursor,
                    PAGE_SIZE,
                )
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
                    state.maturity_job_ids.push(job_id);
                    JobSpec::new(
                        job_id,
                        CreditFacilityMaturityJobConfig {
                            credit_facility_id: *id,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            match self.maturity_spawner.spawn_all_in_op(&mut op, specs).await {
                Ok(_) | Err(JobError::DuplicateId(_)) => {}
                Err(e) => return Err(e.into()),
            }

            state.maturity_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(
                    &mut op,
                    &CreditFacilityEodState::Collecting(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        tracing::info!(
            accrual_jobs = state.accrual_job_ids.len(),
            maturity_jobs = state.maturity_job_ids.len(),
            "Credit facility EOD collection complete, transitioning to tracking"
        );

        // Transition to Tracking state
        let new_state = CreditFacilityEodState::Tracking {
            accrual_job_ids: state.accrual_job_ids,
            maturity_job_ids: state.maturity_job_ids,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn run_tracking(
        &self,
        current_job: CurrentJob,
        accrual_job_ids: Vec<JobId>,
        maturity_job_ids: Vec<JobId>,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let all_job_ids: Vec<JobId> = accrual_job_ids
            .into_iter()
            .chain(maturity_job_ids)
            .collect();

        tracing::info!(
            total_jobs = all_job_ids.len(),
            "Awaiting completion of per-entity credit facility jobs"
        );

        let results = futures::future::try_join_all(
            all_job_ids.iter().map(|id| self.jobs.await_completion(*id)),
        )
        .await?;

        let failed: Vec<_> = all_job_ids
            .iter()
            .zip(results.iter())
            .filter(|(_, state)| *state != &JobTerminalState::Completed)
            .map(|(id, state)| (*id, state.clone()))
            .collect();

        if !failed.is_empty() {
            tracing::error!(
                ?failed,
                "Some credit facility EOD per-entity jobs did not complete successfully"
            );
            return Err(format!("{} credit facility EOD entity jobs failed", failed.len()).into());
        }

        tracing::info!("All credit facility EOD per-entity jobs completed");
        Ok(JobCompletion::Complete)
    }
}

#[async_trait]
impl<E> JobRunner for CreditFacilityEodJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.credit-facility-eod.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<CreditFacilityEodState>()?
            .unwrap_or_default();

        match state {
            CreditFacilityEodState::Collecting(collecting) => {
                self.run_collecting(current_job, collecting).await
            }
            CreditFacilityEodState::Tracking {
                accrual_job_ids,
                maturity_job_ids,
            } => {
                self.run_tracking(current_job, accrual_job_ids, maturity_job_ids)
                    .await
            }
        }
    }
}
