use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use obix::out::OutboxEventMarker;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use job::*;

use crate::{
    eod_process::{EodProcesses, NewEodProcess, error::EodProcessError},
    event::CoreEodEvent,
    phase::{EodContext, EodPhase},
    primitives::*,
};

pub const EOD_PROCESS_MANAGER_JOB: JobType = JobType::new("process.eod.process-manager");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerConfig {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
    pub process_id: EodProcessId,
    pub phase_names: Vec<String>,
}

pub struct EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    jobs: Jobs,
    eod_processes: EodProcesses<E>,
    phases: Arc<Vec<Box<dyn EodPhase>>>,
}

impl<E> EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    pub fn new(
        jobs: &Jobs,
        eod_processes: EodProcesses<E>,
        phases: Arc<Vec<Box<dyn EodPhase>>>,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
            eod_processes,
            phases,
        }
    }
}

impl<E> JobInitializer for EodProcessManagerJobInit<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
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
            phases: Arc::clone(&self.phases),
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
    phases: Arc<Vec<Box<dyn EodPhase>>>,
}

impl<E> EodProcessManagerJobRunner<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    fn find_phase(&self, name: &str) -> Result<&dyn EodPhase, EodProcessError> {
        self.phases
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
            .ok_or_else(|| EodProcessError::PhaseNotRegistered(name.to_string()))
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
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Ensure the entity exists; create it on first invocation.
        let mut op = current_job.begin_op().await?;
        let process = match self
            .eod_processes
            .maybe_find_by_id_in_op(&mut op, self.config.process_id)
            .await?
        {
            Some(p) => {
                drop(op);
                p
            }
            None => {
                let new_process = NewEodProcess::builder()
                    .id(self.config.process_id)
                    .date(self.config.date)
                    .phase_names(self.config.phase_names.clone())
                    .build()?;
                self.eod_processes
                    .create_in_op(&mut op, new_process)
                    .await?;
                return Ok(JobCompletion::RescheduleNowWithOp(op));
            }
        };

        let status = process.status();
        match status {
            EodProcessStatus::Completed | EodProcessStatus::Failed => Ok(JobCompletion::Complete),

            _ => {
                // Check if a phase is currently in progress
                if let Some(current_phase_name) = process.current_phase() {
                    // Await completion of the current phase's job
                    let job_id = process
                        .phase_job_id(current_phase_name)
                        .ok_or(EodProcessError::MissingJobIds)?;

                    let job_ids = [job_id];
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

                    let _ = process.complete_phase(
                        current_phase_name.to_string(),
                        terminals[0].state().into(),
                    )?;

                    self.eod_processes
                        .update_in_op(&mut op, &mut process)
                        .await?;

                    // If terminal (completed all or failed), finish; otherwise reschedule
                    let final_status = process.status();
                    if final_status == EodProcessStatus::Completed
                        || final_status == EodProcessStatus::Failed
                    {
                        Ok(JobCompletion::CompleteWithOp(op))
                    } else {
                        Ok(JobCompletion::RescheduleNowWithOp(op))
                    }
                } else if let Some(next_name) = process.next_phase_name() {
                    // Spawn the next phase
                    let phase = self.find_phase(next_name)?;
                    let phase_job_id = JobId::new();

                    let mut op = current_job.begin_op().await?;

                    let ctx = EodContext {
                        date: self.config.date,
                        closing_time: self.config.closing_time,
                    };
                    phase
                        .spawn_in_op(&mut op, phase_job_id, &ctx)
                        .await
                        .map_err(|e| -> Box<dyn std::error::Error> { e })?;

                    let mut process = self
                        .eod_processes
                        .find_by_id_in_op(&mut op, self.config.process_id)
                        .await?;
                    let _ = process.start_phase(next_name.to_string(), phase_job_id)?;
                    self.eod_processes
                        .update_in_op(&mut op, &mut process)
                        .await?;

                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    // No current phase, no next phase — should not happen unless
                    // entity is in a terminal state (handled above)
                    Ok(JobCompletion::Complete)
                }
            }
        }
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
