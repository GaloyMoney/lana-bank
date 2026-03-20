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
        EodProcess, EodProcesses, JobTerminalState, NewEodProcess, PhaseOutcome,
        error::EodProcessError,
    },
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

impl<E> EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    fn to_entity_terminal(job_terminal: job::JobTerminalState) -> JobTerminalState {
        match job_terminal {
            job::JobTerminalState::Completed => JobTerminalState::Completed,
            job::JobTerminalState::Errored => JobTerminalState::Failed,
            job::JobTerminalState::Cancelled => JobTerminalState::Cancelled,
        }
    }
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
            EodProcessStatus::AwaitingObligationsAndDeposits => {
                self.handle_awaiting_obligations_and_deposits(current_job, &process)
                    .await
            }
            EodProcessStatus::ObligationsAndDepositsComplete => {
                self.handle_obligations_and_deposits_complete(current_job)
                    .await
            }
            EodProcessStatus::AwaitingCreditFacilityEod => {
                self.handle_awaiting_credit_facility_eod(current_job, &process)
                    .await
            }
            EodProcessStatus::Completed
            | EodProcessStatus::Failed
            | EodProcessStatus::Cancelled => Ok(JobCompletion::Complete),
        }
    }
}

impl<E> EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreTimeEvent>,
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
                // Entity created; commit atomically via the job framework
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            Err(EodProcessError::Create(ref e)) if e.was_duplicate() => {
                // Another instance created it — reschedule to load and proceed
                drop(op);
                Ok(JobCompletion::RescheduleNow)
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn spawn_credit_facility_eod_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        process: &mut EodProcess,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let credit_facility_job = JobId::new();

        self.credit_facility_eod_process_spawner
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
            .await?;

        let _ = process.start_credit_facility_eod(credit_facility_job)?;
        Ok(())
    }

    async fn handle_initialized(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let obligation_job = JobId::new();
        let deposit_job = JobId::new();

        let mut op = current_job.begin_op().await?;

        self.obligation_status_process_spawner
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
            .await?;

        self.deposit_activity_process_spawner
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
            .await?;

        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;
        let _ = process.start_obligations_and_deposits(obligation_job, deposit_job)?;
        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;

        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn handle_awaiting_obligations_and_deposits(
        &self,
        mut current_job: CurrentJob,
        process: &EodProcess,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let obligation_job = process
            .obligation_job_id()
            .ok_or("obligation_job_id must be set in AwaitingObligationsAndDeposits")?;
        let deposit_job = process
            .deposit_job_id()
            .ok_or("deposit_job_id must be set in AwaitingObligationsAndDeposits")?;

        let job_ids = [obligation_job, deposit_job];
        let terminals =
            match process_manager::await_job_completions(&mut current_job, &self.jobs, &job_ids)
                .await?
            {
                Some(t) => t,
                None => return Ok(JobCompletion::RescheduleNow),
            };
        let obligation_terminal = Self::to_entity_terminal(terminals[0].state());
        let deposit_terminal = Self::to_entity_terminal(terminals[1].state());

        let mut op = current_job.begin_op().await?;
        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;

        let _ = process.complete_phase1_obligation(obligation_terminal)?;
        let _ = process.complete_phase1_deposit(deposit_terminal)?;

        // lint:allow(service-conditionals)
        match process.evaluate_obligations_and_deposits_outcome() {
            PhaseOutcome::AllSucceeded => {
                self.spawn_credit_facility_eod_in_op(&mut op, &mut process)
                    .await?;
            }
            PhaseOutcome::Failed { reason } => {
                tracing::error!(
                    ?obligation_terminal,
                    ?deposit_terminal,
                    "EOD obligations/deposits failed — manual intervention required"
                );
                let _ = process.mark_failed(reason)?;
            }
        }

        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn handle_obligations_and_deposits_complete(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;

        // lint:allow(service-conditionals)
        match process.evaluate_obligations_and_deposits_outcome() {
            PhaseOutcome::AllSucceeded => {
                self.spawn_credit_facility_eod_in_op(&mut op, &mut process)
                    .await?;
            }
            PhaseOutcome::Failed { reason } => {
                tracing::error!(
                    "EOD handle_obligations_and_deposits_complete: failed — marking failed"
                );
                let _ = process.mark_failed(reason)?;
            }
        }

        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;

        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn handle_awaiting_credit_facility_eod(
        &self,
        mut current_job: CurrentJob,
        process: &EodProcess,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let credit_facility_job = process
            .credit_facility_job_id()
            .ok_or("credit_facility_job_id must be set in AwaitingCreditFacilityEod")?;

        let job_ids = [credit_facility_job];
        let terminals =
            match process_manager::await_job_completions(&mut current_job, &self.jobs, &job_ids)
                .await?
            {
                Some(t) => t,
                None => return Ok(JobCompletion::RescheduleNow),
            };
        let credit_facility_terminal = Self::to_entity_terminal(terminals[0].state());

        let mut op = current_job.begin_op().await?;
        let mut process = self
            .eod_processes
            .find_by_id_in_op(&mut op, self.config.process_id)
            .await?;

        let _ = process.complete_phase2_credit_facility(credit_facility_terminal)?;

        // lint:allow(service-conditionals)
        match process.evaluate_credit_facility_eod_outcome() {
            PhaseOutcome::AllSucceeded => {
                let _ = process.mark_completed()?;
            }
            PhaseOutcome::Failed { reason } => {
                tracing::error!(
                    ?credit_facility_terminal,
                    "EOD credit-facility-eod failed — manual intervention required"
                );
                let _ = process.mark_failed(reason)?;
            }
        }

        self.eod_processes
            .update_in_op(&mut op, &mut process)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
