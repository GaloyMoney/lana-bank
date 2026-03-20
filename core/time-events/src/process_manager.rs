use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use es_entity::Idempotent;
use obix::out::OutboxEventMarker;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use job::*;

use crate::{
    credit_facility_eod_process::{
        CreditFacilityEodProcessConfig, CreditFacilityEodProcessSpawner,
    },
    deposit_activity_process::{DepositActivityProcessConfig, DepositActivityProcessSpawner},
    eod_process::{EodProcesses, NewEodProcess},
    event::CoreTimeEvent,
    obligation_status_process::{ObligationStatusProcessConfig, ObligationStatusProcessSpawner},
    primitives::*,
};

pub const EOD_PROCESS_MANAGER_JOB: JobType = JobType::new("process.eod.process-manager");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerConfig {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
    pub process_id: EodProcessId,
}

pub struct EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    jobs: Jobs,
    eod_processes: EodProcesses<E>,
    obligation_status_process_spawner: ObligationStatusProcessSpawner,
    deposit_activity_process_spawner: DepositActivityProcessSpawner,
    credit_facility_eod_process_spawner: CreditFacilityEodProcessSpawner,
}

impl<E> EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(
        jobs: &Jobs,
        eod_processes: EodProcesses<E>,
        obligation_status_process_spawner: ObligationStatusProcessSpawner,
        deposit_activity_process_spawner: DepositActivityProcessSpawner,
        credit_facility_eod_process_spawner: CreditFacilityEodProcessSpawner,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
            eod_processes,
            obligation_status_process_spawner,
            deposit_activity_process_spawner,
            credit_facility_eod_process_spawner,
        }
    }
}

impl<E> JobInitializer for EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    type Config = EodProcessManagerConfig;

    fn job_type(&self) -> JobType {
        EOD_PROCESS_MANAGER_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EodProcessManagerJobRunner {
            config: job.config()?,
            jobs: self.jobs.clone(),
            eod_processes: self.eod_processes.clone(),
            obligation_status_process_spawner: self.obligation_status_process_spawner.clone(),
            deposit_activity_process_spawner: self.deposit_activity_process_spawner.clone(),
            credit_facility_eod_process_spawner: self.credit_facility_eod_process_spawner.clone(),
        }))
    }
}

/// Internal state for the EOD process manager, persisted via job execution
/// state. The PM dispatches on its own state rather than querying the entity's
/// status, following the same pattern as other process managers in the codebase.
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
enum EodProcessManagerState {
    /// Create the EodProcess entity if it does not already exist.
    #[default]
    Initializing,
    /// Spawn obligation-status and deposit-activity child jobs and record
    /// them on the entity.
    SpawningObligationsAndDeposits,
    /// Wait for the obligation and deposit child jobs to complete.
    AwaitingObligationsAndDeposits {
        obligation_job: JobId,
        deposit_job: JobId,
    },
    /// Wait for the credit-facility EOD child job to complete.
    AwaitingCreditFacilityEod { credit_facility_job: JobId },
    /// Terminal: the process has finished (completed or failed).
    Done,
}

struct EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    config: EodProcessManagerConfig,
    jobs: Jobs,
    eod_processes: EodProcesses<E>,
    obligation_status_process_spawner: ObligationStatusProcessSpawner,
    deposit_activity_process_spawner: DepositActivityProcessSpawner,
    credit_facility_eod_process_spawner: CreditFacilityEodProcessSpawner,
}

#[async_trait]
impl<E> JobRunner for EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.process-manager.run",
        skip(self, current_job),
        fields(date = %self.config.date, process_id = %self.config.process_id)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<EodProcessManagerState>()?
            .unwrap_or_default();

        match state {
            EodProcessManagerState::Initializing => {
                let new_process = NewEodProcess::builder()
                    .id(self.config.process_id)
                    .date(self.config.date)
                    .build()?;
                let mut op = current_job.begin_op().await?;
                self.eod_processes
                    .create_in_op(&mut op, new_process)
                    .await?;
                current_job
                    .update_execution_state_in_op(
                        &mut op,
                        &EodProcessManagerState::SpawningObligationsAndDeposits,
                    )
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }

            EodProcessManagerState::SpawningObligationsAndDeposits => {
                let obligation_job = JobId::new();
                let deposit_job = JobId::new();

                let mut op = current_job.begin_op().await?;

                self.obligation_status_process_spawner
                    .spawn_in_op(
                        &mut op,
                        obligation_job,
                        ObligationStatusProcessConfig {
                            date: self.config.date,
                        },
                    )
                    .await?;

                self.deposit_activity_process_spawner
                    .spawn_in_op(
                        &mut op,
                        deposit_job,
                        DepositActivityProcessConfig {
                            date: self.config.date,
                            closing_time: self.config.closing_time,
                        },
                    )
                    .await?;

                let mut process = self
                    .eod_processes
                    .find_by_id_in_op(&mut op, self.config.process_id)
                    .await?;
                let _ = process.start_obligations_and_deposits(obligation_job, deposit_job)?;
                self.eod_processes
                    .update_in_op(&mut op, &mut process)
                    .await?;

                current_job
                    .update_execution_state_in_op(
                        &mut op,
                        &EodProcessManagerState::AwaitingObligationsAndDeposits {
                            obligation_job,
                            deposit_job,
                        },
                    )
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }

            EodProcessManagerState::AwaitingObligationsAndDeposits {
                obligation_job,
                deposit_job,
            } => {
                let job_ids = [obligation_job, deposit_job];
                let terminals = match process_manager::await_job_completions(
                    &mut current_job,
                    &self.jobs,
                    &job_ids,
                )
                .await?
                {
                    Some(t) => t,
                    None => return Ok(JobCompletion::RescheduleNow),
                };

                let credit_facility_job = JobId::new();
                let mut op = current_job.begin_op().await?;
                let mut process = self
                    .eod_processes
                    .find_by_id_in_op(&mut op, self.config.process_id)
                    .await?;

                let advanced = process.complete_obligations_and_deposits(
                    terminals[0].state().into(),
                    terminals[1].state().into(),
                    credit_facility_job,
                )?;

                let next_state = if let Idempotent::Executed(true) = advanced {
                    self.credit_facility_eod_process_spawner
                        .spawn_in_op(
                            &mut op,
                            credit_facility_job,
                            CreditFacilityEodProcessConfig {
                                date: self.config.date,
                            },
                        )
                        .await?;
                    EodProcessManagerState::AwaitingCreditFacilityEod {
                        credit_facility_job,
                    }
                } else {
                    EodProcessManagerState::Done
                };

                self.eod_processes
                    .update_in_op(&mut op, &mut process)
                    .await?;
                current_job
                    .update_execution_state_in_op(&mut op, &next_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }

            EodProcessManagerState::AwaitingCreditFacilityEod {
                credit_facility_job,
            } => {
                let job_ids = [credit_facility_job];
                let terminals = match process_manager::await_job_completions(
                    &mut current_job,
                    &self.jobs,
                    &job_ids,
                )
                .await?
                {
                    Some(t) => t,
                    None => return Ok(JobCompletion::RescheduleNow),
                };

                let mut op = current_job.begin_op().await?;
                let mut process = self
                    .eod_processes
                    .find_by_id_in_op(&mut op, self.config.process_id)
                    .await?;

                let _ = process.complete_credit_facility_eod(terminals[0].state().into())?;

                self.eod_processes
                    .update_in_op(&mut op, &mut process)
                    .await?;
                Ok(JobCompletion::CompleteWithOp(op))
            }

            EodProcessManagerState::Done => Ok(JobCompletion::Complete),
        }
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
