use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
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
    eod_process::{
        EodPhase, EodProcess, EodProcesses, JobTerminalState, NewEodProcess, error::EodProcessError,
    },
    obligation_status_process::{ObligationStatusProcessConfig, ObligationStatusProcessSpawner},
    primitives::*,
    public::CoreEodEvent,
};

pub const EOD_PROCESS_MANAGER_JOB_TYPE: JobType = JobType::new("task.eod.process-manager");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerConfig {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
    pub process_id: EodProcessId,
}

pub struct EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    jobs: Jobs,
    eod_processes: EodProcesses<E>,
    obligation_spawner: ObligationStatusProcessSpawner,
    deposit_spawner: DepositActivityProcessSpawner,
    credit_facility_spawner: CreditFacilityEodProcessSpawner,
}

impl<E> EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    pub fn new(
        jobs: &Jobs,
        eod_processes: EodProcesses<E>,
        obligation_spawner: ObligationStatusProcessSpawner,
        deposit_spawner: DepositActivityProcessSpawner,
        credit_facility_spawner: CreditFacilityEodProcessSpawner,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
            eod_processes,
            obligation_spawner,
            deposit_spawner,
            credit_facility_spawner,
        }
    }
}

impl<E> JobInitializer for EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    type Config = EodProcessManagerConfig;

    fn job_type(&self) -> JobType {
        EOD_PROCESS_MANAGER_JOB_TYPE
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
            obligation_spawner: self.obligation_spawner.clone(),
            deposit_spawner: self.deposit_spawner.clone(),
            credit_facility_spawner: self.credit_facility_spawner.clone(),
        }))
    }
}

struct EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    config: EodProcessManagerConfig,
    jobs: Jobs,
    eod_processes: EodProcesses<E>,
    obligation_spawner: ObligationStatusProcessSpawner,
    deposit_spawner: DepositActivityProcessSpawner,
    credit_facility_spawner: CreditFacilityEodProcessSpawner,
}

impl<E> EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    fn to_entity_terminal(job_terminal: job::JobTerminalState) -> JobTerminalState {
        match job_terminal {
            job::JobTerminalState::Completed => JobTerminalState::Completed,
            job::JobTerminalState::Failed => JobTerminalState::Failed,
            job::JobTerminalState::Cancelled => JobTerminalState::Cancelled,
        }
    }
}

#[async_trait]
impl<E> JobRunner for EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.process-manager.run",
        skip(self, current_job),
        fields(date = %self.config.date, process_id = %self.config.process_id)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let process = match self.eod_processes.find_by_id(self.config.process_id).await {
            Ok(p) => p,
            Err(EodProcessError::Find(ref e)) if e.was_not_found() => {
                // First run — create entity
                return self.create_entity_and_start(current_job).await;
            }
            Err(e) => return Err(e.into()),
        };

        match process.status() {
            EodProcessStatus::Initialized => self.handle_initialized(current_job).await,
            EodProcessStatus::AwaitingPhase1 => {
                self.handle_awaiting_phase1(current_job, &process).await
            }
            EodProcessStatus::Phase1Complete => self.handle_phase1_complete(current_job).await,
            EodProcessStatus::AwaitingPhase2 => {
                self.handle_awaiting_phase2(current_job, &process).await
            }
            EodProcessStatus::Completed
            | EodProcessStatus::Failed
            | EodProcessStatus::Cancelled => Ok(JobCompletion::Complete),
        }
    }
}

impl<E> EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    async fn create_entity_and_start(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let process_id = self.config.process_id;
        let new_process = NewEodProcess::builder()
            .id(process_id)
            .date(self.config.date)
            .build()?;

        let mut op = current_job.begin_op().await?;
        match self.eod_processes.create_in_op(&mut op, new_process).await {
            Ok(_) => {
                op.commit().await?;
                // Entity created; reschedule to proceed with orchestration
                Ok(JobCompletion::RescheduleNow)
            }
            Err(EodProcessError::Create(ref e)) if e.was_duplicate() => {
                // Another instance created it — reschedule to load and proceed
                drop(op);
                Ok(JobCompletion::RescheduleNow)
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn spawn_phase2(
        &self,
        op: &mut es_entity::DbOp<'_>,
        process: &mut EodProcess,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let credit_facility_job = JobId::new();

        match self
            .credit_facility_spawner
            .spawn_all_in_op(
                op,
                vec![
                    JobSpec::new(
                        credit_facility_job,
                        CreditFacilityEodProcessConfig {
                            date: self.config.date,
                        },
                    )
                    .queue_id("eod-credit-facility".to_string()),
                ],
            )
            .await
        {
            Ok(_) | Err(job::error::JobError::DuplicateId(_)) => {}
            Err(e) => return Err(e.into()),
        }

        process.start_phase2(credit_facility_job)?;
        Ok(())
    }

    async fn handle_initialized(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let obligation_job = JobId::new();
        let deposit_job = JobId::new();

        let mut op = current_job.begin_op().await?;

        match self
            .obligation_spawner
            .spawn_all_in_op(
                &mut op,
                vec![
                    JobSpec::new(
                        obligation_job,
                        ObligationStatusProcessConfig {
                            date: self.config.date,
                        },
                    )
                    .queue_id("eod-obligation-status".to_string()),
                ],
            )
            .await
        {
            Ok(_) | Err(job::error::JobError::DuplicateId(_)) => {}
            Err(e) => return Err(e.into()),
        }

        match self
            .deposit_spawner
            .spawn_all_in_op(
                &mut op,
                vec![
                    JobSpec::new(
                        deposit_job,
                        DepositActivityProcessConfig {
                            date: self.config.date,
                            closing_time: self.config.closing_time,
                        },
                    )
                    .queue_id("eod-deposit-activity".to_string()),
                ],
            )
            .await
        {
            Ok(_) | Err(job::error::JobError::DuplicateId(_)) => {}
            Err(e) => return Err(e.into()),
        }

        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;
        process.start_phase1(obligation_job, deposit_job)?;
        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;

        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn handle_awaiting_phase1(
        &self,
        current_job: CurrentJob,
        process: &EodProcess,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let obligation_job = process
            .obligation_job_id()
            .ok_or("obligation_job_id must be set in AwaitingPhase1")?;
        let deposit_job = process
            .deposit_job_id()
            .ok_or("deposit_job_id must be set in AwaitingPhase1")?;

        // Check for cancellation before awaiting children
        if current_job.cancellation_requested() {
            let _ = self.jobs.cancel(obligation_job).await;
            let _ = self.jobs.cancel(deposit_job).await;

            let mut op = current_job.begin_op().await?;
            let mut process = self
                .eod_processes
                .find_by_id_in_op(&mut op, self.config.process_id)
                .await?;
            process.request_cancellation()?;
            process.mark_cancelled()?;
            self.eod_processes
                .update_in_op(&mut op, &mut process)
                .await?;
            return Ok(JobCompletion::RescheduleNowWithOp(op));
        }

        let (obligation_result, deposit_result) = tokio::select! {
            results = async {
                tokio::join!(
                    self.jobs.await_completion(obligation_job),
                    self.jobs.await_completion(deposit_job),
                )
            } => results,
            _ = current_job.shutdown_requested() => {
                return Ok(JobCompletion::RescheduleIn(Duration::ZERO));
            }
        };
        let obligation_terminal = Self::to_entity_terminal(obligation_result?);
        let deposit_terminal = Self::to_entity_terminal(deposit_result?);

        let mut op = current_job.begin_op().await?;
        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;

        process.complete_phase1_obligation(obligation_terminal)?;
        process.complete_phase1_deposit(deposit_terminal)?;

        if obligation_terminal == JobTerminalState::Completed
            && deposit_terminal == JobTerminalState::Completed
        {
            self.spawn_phase2(&mut op, &mut process).await?;
        } else {
            tracing::error!(
                phase = 1,
                ?obligation_terminal,
                ?deposit_terminal,
                "EOD process manager failed — manual intervention required"
            );
            process.mark_failed(
                EodPhase::Phase1,
                format!(
                    "Phase 1 children failed: obligation={obligation_terminal:?}, deposit={deposit_terminal:?}"
                ),
            )?;
        }

        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn handle_phase1_complete(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;

        // Only spawn Phase 2 if both Phase 1 children actually succeeded
        let obligation_ok =
            process.phase1_obligation_terminal() == Some(JobTerminalState::Completed);
        let deposit_ok = process.phase1_deposit_terminal() == Some(JobTerminalState::Completed);

        if obligation_ok && deposit_ok {
            self.spawn_phase2(&mut op, &mut process).await?;
        } else {
            let reason = format!(
                "Phase 1 children failed: obligation={:?}, deposit={:?}",
                process.phase1_obligation_terminal(),
                process.phase1_deposit_terminal()
            );
            tracing::error!(
                phase = 1,
                ?obligation_ok,
                ?deposit_ok,
                "EOD handle_phase1_complete: phase 1 failed — marking failed"
            );
            process.mark_failed(EodPhase::Phase1, reason)?;
        }

        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;

        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn handle_awaiting_phase2(
        &self,
        current_job: CurrentJob,
        process: &EodProcess,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let credit_facility_job = process
            .credit_facility_job_id()
            .ok_or("credit_facility_job_id must be set in AwaitingPhase2")?;

        // Check for cancellation before awaiting children
        if current_job.cancellation_requested() {
            let _ = self.jobs.cancel(credit_facility_job).await;

            let mut op = current_job.begin_op().await?;
            let mut process = self
                .eod_processes
                .find_by_id_in_op(&mut op, self.config.process_id)
                .await?;
            process.request_cancellation()?;
            process.mark_cancelled()?;
            self.eod_processes
                .update_in_op(&mut op, &mut process)
                .await?;
            return Ok(JobCompletion::RescheduleNowWithOp(op));
        }

        let credit_facility_terminal = tokio::select! {
            result = self.jobs.await_completion(credit_facility_job) => {
                Self::to_entity_terminal(result?)
            }
            _ = current_job.shutdown_requested() => {
                return Ok(JobCompletion::RescheduleIn(Duration::ZERO));
            }
        };

        let mut op = current_job.begin_op().await?;
        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;

        process.complete_phase2_credit_facility(credit_facility_terminal)?;

        if credit_facility_terminal == JobTerminalState::Completed {
            process.mark_completed()?;
        } else {
            tracing::error!(
                phase = 2,
                ?credit_facility_terminal,
                "EOD process manager failed — manual intervention required"
            );
            process.mark_failed(
                EodPhase::Phase2,
                format!("Phase 2 credit-facility child failed: {credit_facility_terminal:?}"),
            )?;
        }

        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
