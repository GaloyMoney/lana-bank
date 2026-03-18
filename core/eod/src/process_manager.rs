use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use job::{error::JobError, *};

use strum::Display;

use crate::{
    credit_facility_eod_process::{
        CreditFacilityEodProcessConfig, CreditFacilityEodProcessSpawner,
    },
    deposit_activity_process::{DepositActivityProcessConfig, DepositActivityProcessSpawner},
    job_id,
    obligation_transition_process::{
        ObligationTransitionProcessConfig, ObligationTransitionProcessSpawner,
    },
};

pub const EOD_PROCESS_MANAGER_JOB_TYPE: JobType = JobType::new("task.eod.process-manager");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerConfig {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
}

/// Structured result set on the process manager job upon completion.
///
/// Count fields are intentionally omitted — the parent PM does not have
/// access to child result data yet. Counts will be added when
/// child set_result/get_result propagation is available.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerResult {
    pub date: NaiveDate,
    pub phase1_obligation: JobTerminalState,
    pub phase1_deposit: JobTerminalState,
    /// None if Phase 2 never started (e.g. Phase 1 failed).
    pub phase2_credit_facility: Option<JobTerminalState>,
}

/// Identifies which child process manager is being referenced in
/// Failed / Cancelling states, avoiding stringly-typed matching.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[serde(rename_all = "camelCase")]
pub(crate) enum EodChildProcess {
    ObligationTransition,
    DepositActivity,
    CreditFacilityEod,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) enum EodProcessState {
    #[default]
    SpawningPhase1,
    AwaitingPhase1 {
        obligation_job: JobId,
        deposit_job: JobId,
    },
    SpawningPhase2 {
        phase1_obligation: JobTerminalState,
        phase1_deposit: JobTerminalState,
    },
    AwaitingPhase2 {
        phase1_obligation: JobTerminalState,
        phase1_deposit: JobTerminalState,
        credit_facility_job: JobId,
    },
    Completed {
        result: EodProcessManagerResult,
    },
    Failed {
        phase: u8,
        failed: Vec<(EodChildProcess, JobTerminalState)>,
    },
    Cancelling {
        phase: u8,
        children: Vec<(EodChildProcess, JobId)>,
    },
}

pub struct EodProcessManagerJobInit {
    jobs: Jobs,
    obligation_spawner: ObligationTransitionProcessSpawner,
    deposit_spawner: DepositActivityProcessSpawner,
    credit_facility_spawner: CreditFacilityEodProcessSpawner,
}

impl EodProcessManagerJobInit {
    pub fn new(
        jobs: &Jobs,
        obligation_spawner: ObligationTransitionProcessSpawner,
        deposit_spawner: DepositActivityProcessSpawner,
        credit_facility_spawner: CreditFacilityEodProcessSpawner,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
            obligation_spawner,
            deposit_spawner,
            credit_facility_spawner,
        }
    }
}

impl JobInitializer for EodProcessManagerJobInit {
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
            obligation_spawner: self.obligation_spawner.clone(),
            deposit_spawner: self.deposit_spawner.clone(),
            credit_facility_spawner: self.credit_facility_spawner.clone(),
        }))
    }
}

struct EodProcessManagerJobRunner {
    config: EodProcessManagerConfig,
    jobs: Jobs,
    obligation_spawner: ObligationTransitionProcessSpawner,
    deposit_spawner: DepositActivityProcessSpawner,
    credit_facility_spawner: CreditFacilityEodProcessSpawner,
}

#[async_trait]
impl JobRunner for EodProcessManagerJobRunner {
    #[record_error_severity]
    #[instrument(
        name = "eod.process-manager.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<EodProcessState>()?
            .unwrap_or_default();

        match state {
            EodProcessState::SpawningPhase1 => {
                let obligation_job =
                    job_id::eod_child_id(&self.config.date, "obligation-transition");
                let deposit_job = job_id::eod_child_id(&self.config.date, "deposit-activity");

                let mut op = current_job.begin_op().await?;

                match self
                    .obligation_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![JobSpec::new(
                            obligation_job,
                            ObligationTransitionProcessConfig {
                                date: self.config.date,
                            },
                        )
                        .queue_id("eod-obligation-transition".to_string())],
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                match self
                    .deposit_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![JobSpec::new(
                            deposit_job,
                            DepositActivityProcessConfig {
                                date: self.config.date,
                                closing_time: self.config.closing_time,
                            },
                        )
                        .queue_id("eod-deposit-activity".to_string())],
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                let new_state = EodProcessState::AwaitingPhase1 {
                    obligation_job,
                    deposit_job,
                };
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            EodProcessState::AwaitingPhase1 {
                obligation_job,
                deposit_job,
            } => {
                // Check for cancellation before awaiting children
                if current_job.cancellation_requested() {
                    let children = vec![
                        (EodChildProcess::ObligationTransition, obligation_job),
                        (EodChildProcess::DepositActivity, deposit_job),
                    ];
                    for (_, child_id) in &children {
                        let _ = self.jobs.cancel(*child_id).await;
                    }
                    let new_state = EodProcessState::Cancelling { phase: 1, children };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    return Ok(JobCompletion::RescheduleNowWithOp(op));
                }

                let (obligation_result, deposit_result) = tokio::join!(
                    self.jobs.await_completion(obligation_job),
                    self.jobs.await_completion(deposit_job),
                );
                let obligation_terminal = obligation_result?;
                let deposit_terminal = deposit_result?;

                let mut failed = Vec::new();
                if obligation_terminal != JobTerminalState::Completed {
                    failed.push((EodChildProcess::ObligationTransition, obligation_terminal));
                }
                if deposit_terminal != JobTerminalState::Completed {
                    failed.push((EodChildProcess::DepositActivity, deposit_terminal));
                }

                if !failed.is_empty() {
                    tracing::error!(
                        phase = 1,
                        ?failed,
                        "EOD process manager failed — manual intervention required"
                    );
                    let new_state = EodProcessState::Failed { phase: 1, failed };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let new_state = EodProcessState::SpawningPhase2 {
                        phase1_obligation: obligation_terminal,
                        phase1_deposit: deposit_terminal,
                    };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
            }
            EodProcessState::SpawningPhase2 {
                phase1_obligation,
                phase1_deposit,
            } => {
                let credit_facility_job =
                    job_id::eod_child_id(&self.config.date, "credit-facility");

                let mut op = current_job.begin_op().await?;

                match self
                    .credit_facility_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![JobSpec::new(
                            credit_facility_job,
                            CreditFacilityEodProcessConfig {
                                date: self.config.date,
                            },
                        )
                        .queue_id("eod-credit-facility".to_string())],
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                let new_state = EodProcessState::AwaitingPhase2 {
                    phase1_obligation,
                    phase1_deposit,
                    credit_facility_job,
                };
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            EodProcessState::AwaitingPhase2 {
                phase1_obligation,
                phase1_deposit,
                credit_facility_job,
            } => {
                // Check for cancellation before awaiting children
                if current_job.cancellation_requested() {
                    let children = vec![(EodChildProcess::CreditFacilityEod, credit_facility_job)];
                    for (_, child_id) in &children {
                        let _ = self.jobs.cancel(*child_id).await;
                    }
                    let new_state = EodProcessState::Cancelling { phase: 2, children };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    return Ok(JobCompletion::RescheduleNowWithOp(op));
                }

                let credit_facility_terminal =
                    self.jobs.await_completion(credit_facility_job).await?;

                if credit_facility_terminal != JobTerminalState::Completed {
                    let failed =
                        vec![(EodChildProcess::CreditFacilityEod, credit_facility_terminal)];
                    tracing::error!(
                        phase = 2,
                        ?failed,
                        "EOD process manager failed — manual intervention required"
                    );
                    let new_state = EodProcessState::Failed { phase: 2, failed };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let result = EodProcessManagerResult {
                        date: self.config.date,
                        phase1_obligation,
                        phase1_deposit,
                        phase2_credit_facility: Some(credit_facility_terminal),
                    };
                    // Checkpoint to Completed first; set_result runs on the
                    // next iteration from the Completed arm, making this
                    // crash-safe: if we crash between checkpoint and
                    // set_result the PM simply retries set_result.
                    let new_state = EodProcessState::Completed { result };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
            }
            EodProcessState::Completed { result } => {
                // Idempotent: set_result may have already been called before
                // a crash; calling it again is safe.
                current_job.set_result(&result).await?;
                Ok(JobCompletion::Complete)
            }
            EodProcessState::Failed { phase, failed } => {
                // Build a partial result from the failure info so callers
                // can inspect what happened.
                let (obligation_state, deposit_state, credit_state) = match phase {
                    1 => {
                        let obligation = failed
                            .iter()
                            .find(|(n, _)| *n == EodChildProcess::ObligationTransition)
                            .map(|(_, s)| s.clone())
                            .unwrap_or(JobTerminalState::Completed);
                        let deposit = failed
                            .iter()
                            .find(|(n, _)| *n == EodChildProcess::DepositActivity)
                            .map(|(_, s)| s.clone())
                            .unwrap_or(JobTerminalState::Completed);
                        // Phase 2 never ran
                        (obligation, deposit, None)
                    }
                    _ => {
                        let credit = failed
                            .iter()
                            .find(|(n, _)| *n == EodChildProcess::CreditFacilityEod)
                            .map(|(_, s)| s.clone())
                            .unwrap_or(JobTerminalState::Completed);
                        (
                            JobTerminalState::Completed,
                            JobTerminalState::Completed,
                            Some(credit),
                        )
                    }
                };

                let result = EodProcessManagerResult {
                    date: self.config.date,
                    phase1_obligation: obligation_state,
                    phase1_deposit: deposit_state,
                    phase2_credit_facility: credit_state,
                };
                current_job.set_result(&result).await?;
                Ok(JobCompletion::Complete)
            }
            EodProcessState::Cancelling { phase, children } => {
                tracing::warn!(
                    phase,
                    children = ?children.iter().map(|(n, _)| n.to_string()).collect::<Vec<_>>(),
                    "EOD process manager cancelling children"
                );
                for (_, child_id) in &children {
                    let _ = self.jobs.cancel(*child_id).await;
                }

                // Report all children as Cancelled so callers can inspect
                // the terminal state, mirroring Completed / Failed.
                let result = EodProcessManagerResult {
                    date: self.config.date,
                    phase1_obligation: JobTerminalState::Cancelled,
                    phase1_deposit: JobTerminalState::Cancelled,
                    phase2_credit_facility: if phase >= 2 {
                        Some(JobTerminalState::Cancelled)
                    } else {
                        None
                    },
                };
                current_job.set_result(&result).await?;
                Ok(JobCompletion::Complete)
            }
        }
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
