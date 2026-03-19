use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use governance::GovernanceEvent;
use job::{error::JobError, *};
use obix::out::OutboxEventMarker;

use core_custody::CoreCustodyEvent;
use core_price::CorePriceEvent;
use core_time_events::credit_facility_eod_process::{
    CREDIT_FACILITY_EOD_PROCESS_JOB_TYPE, CreditFacilityEodProcessConfig,
};

use super::credit_facility_maturity::{
    CreditFacilityMaturityJobConfig, CreditFacilityMaturityJobSpawner,
};
use super::interest_accrual_process::{
    InterestAccrualProcessConfig, InterestAccrualProcessSpawner,
};
use crate::{CoreCreditEvent, CreditFacilityId, credit_facility::CreditFacilityRepo};

const PAGE_SIZE: i64 = 100;

pub struct CreditFacilityEodProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    jobs: Jobs,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    interest_accrual_process_spawner: InterestAccrualProcessSpawner,
    maturity_spawner: CreditFacilityMaturityJobSpawner,
}

impl<E> CreditFacilityEodProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        jobs: &Jobs,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        interest_accrual_process_spawner: InterestAccrualProcessSpawner,
        maturity_spawner: CreditFacilityMaturityJobSpawner,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
            credit_facility_repo,
            interest_accrual_process_spawner,
            maturity_spawner,
        }
    }
}

impl<E> JobInitializer for CreditFacilityEodProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = CreditFacilityEodProcessConfig;

    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_EOD_PROCESS_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityEodProcessRunner {
            config: job.config()?,
            jobs: self.jobs.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            interest_accrual_process_spawner: self.interest_accrual_process_spawner.clone(),
            maturity_spawner: self.maturity_spawner.clone(),
        }))
    }
}

struct CreditFacilityEodProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: CreditFacilityEodProcessConfig,
    jobs: Jobs,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    interest_accrual_process_spawner: InterestAccrualProcessSpawner,
    maturity_spawner: CreditFacilityMaturityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CreditFacilityEodState {
    #[default]
    SpawningAccrualAndMaturityJobs(SpawningAccrualAndMaturityJobsState),
    AwaitingAccrualsAndMaturities {
        pending_accrual_jobs: Vec<JobId>,
        pending_maturity_jobs: Vec<JobId>,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpawningAccrualAndMaturityJobsState {
    accrual_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    maturity_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    pending_accrual_jobs: Vec<JobId>,
    pending_maturity_jobs: Vec<JobId>,
    accrual_done: bool,
}

impl<E> CreditFacilityEodProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    /// Step 1: Query facilities eligible for accrual and maturity (paginated)
    /// and spawn per-facility jobs for each. Transitions to
    /// AwaitingAccrualsAndMaturities when all pages are processed.
    async fn spawn_accrual_and_maturity_jobs(
        &self,
        mut current_job: CurrentJob,
        mut state: SpawningAccrualAndMaturityJobsState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Phase: Spawn accrual jobs for eligible facilities
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
                            &CreditFacilityEodState::SpawningAccrualAndMaturityJobs(state.clone()),
                        )
                        .await?;
                    op.commit().await?;
                    break;
                }

                let specs: Vec<_> = rows
                    .iter()
                    .map(|(id, _)| {
                        let job_id = JobId::new();
                        state.pending_accrual_jobs.push(job_id);
                        JobSpec::new(
                            job_id,
                            InterestAccrualProcessConfig {
                                credit_facility_id: *id,
                                date: self.config.date,
                            },
                        )
                        .queue_id(id.to_string())
                    })
                    .collect();

                match self
                    .interest_accrual_process_spawner
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
                        &CreditFacilityEodState::SpawningAccrualAndMaturityJobs(state.clone()),
                    )
                    .await?;
                op.commit().await?;
            }
        }

        // Phase: Spawn maturity jobs for facilities ready to mature
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
                    let job_id = JobId::new();
                    state.pending_maturity_jobs.push(job_id);
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
                    &CreditFacilityEodState::SpawningAccrualAndMaturityJobs(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        let total = state.pending_accrual_jobs.len() + state.pending_maturity_jobs.len();
        tracing::info!(
            accruals = state.pending_accrual_jobs.len(),
            maturities = state.pending_maturity_jobs.len(),
            total,
            "Credit facility EOD spawning complete, transitioning to awaiting"
        );

        let new_state = CreditFacilityEodState::AwaitingAccrualsAndMaturities {
            pending_accrual_jobs: state.pending_accrual_jobs,
            pending_maturity_jobs: state.pending_maturity_jobs,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    /// Step 2: Await completion of all spawned accrual and maturity jobs using
    /// Jobs.await_completion. This avoids the multi-period hang that occurred
    /// with outbox event streaming, since InterestAccrualProcess always
    /// completes even when a facility has remaining accrual periods.
    async fn await_accrual_and_maturity_completions(
        &self,
        current_job: CurrentJob,
        pending_accrual_jobs: Vec<JobId>,
        pending_maturity_jobs: Vec<JobId>,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if pending_accrual_jobs.is_empty() && pending_maturity_jobs.is_empty() {
            tracing::info!("No credit facilities to track, completing immediately");
            return Ok(JobCompletion::Complete);
        }

        tracing::info!(
            remaining_accruals = pending_accrual_jobs.len(),
            remaining_maturities = pending_maturity_jobs.len(),
            "Awaiting credit facility EOD job completions"
        );

        let futures: Vec<_> = pending_accrual_jobs
            .iter()
            .chain(pending_maturity_jobs.iter())
            .map(|job_id| self.jobs.await_completion(*job_id))
            .collect();

        let results = tokio::select! {
            results = futures::future::join_all(futures) => results,
            _ = current_job.shutdown_requested() => {
                return Ok(JobCompletion::RescheduleIn(Duration::ZERO));
            }
        };

        let mut failed_count = 0usize;
        for result in &results {
            let terminal = result.as_ref().map_err(|e| e.to_string())?;
            if *terminal != job::JobTerminalState::Completed {
                failed_count += 1;
            }
        }

        if failed_count > 0 {
            return Err(format!(
                "{failed_count} of {} credit facility EOD child jobs did not complete successfully",
                results.len()
            )
            .into());
        }

        tracing::info!("All credit facility EOD per-entity jobs completed");
        Ok(JobCompletion::Complete)
    }
}

#[async_trait]
impl<E> JobRunner for CreditFacilityEodProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.credit-facility-eod-process.run",
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
            CreditFacilityEodState::SpawningAccrualAndMaturityJobs(spawning) => {
                self.spawn_accrual_and_maturity_jobs(current_job, spawning)
                    .await
            }
            CreditFacilityEodState::AwaitingAccrualsAndMaturities {
                pending_accrual_jobs,
                pending_maturity_jobs,
            } => {
                self.await_accrual_and_maturity_completions(
                    current_job,
                    pending_accrual_jobs,
                    pending_maturity_jobs,
                )
                .await
            }
        }
    }
}
