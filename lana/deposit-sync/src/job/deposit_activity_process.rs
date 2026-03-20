use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountId,
    GovernanceAction, GovernanceObject,
};
use core_time_events::deposit_activity_process::{
    DEPOSIT_ACTIVITY_PROCESS_JOB_TYPE, DepositActivityProcessConfig,
};
use governance::GovernanceEvent;
use job::*;
use obix::out::{Outbox, OutboxEventMarker};

use super::evaluate_deposit_account_activity::{
    EvaluateDepositAccountActivityConfig, EvaluateDepositAccountActivityJobSpawner,
};

const PAGE_SIZE: i64 = 100;

pub struct DepositActivityProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

impl<Perms, E> DepositActivityProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        jobs: &Jobs,
        deposits: &CoreDeposit<Perms, E>,
        evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            jobs: jobs.clone(),
            deposits: deposits.clone(),
            evaluate_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for DepositActivityProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<CoreCustomerAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<CustomerObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    type Config = DepositActivityProcessConfig;

    fn job_type(&self) -> JobType {
        DEPOSIT_ACTIVITY_PROCESS_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DepositActivityProcessRunner {
            config: job.config()?,
            _outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
            deposits: self.deposits.clone(),
            evaluate_spawner: self.evaluate_spawner.clone(),
        }))
    }
}

struct DepositActivityProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    config: DepositActivityProcessConfig,
    // Kept for future use when a public DepositActivityEvaluated event is added
    _outbox: Outbox<E>,
    jobs: Jobs,
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DepositActivityState {
    #[default]
    SpawningActivityJobs(SpawningActivityJobsState),
    AwaitingActivityCompletion {
        // Vec (not HashSet) because await_completion needs ordered job IDs,
        // unlike the other PMs which use HashSet for O(1) event-stream lookups.
        pending_jobs: Vec<(DepositAccountId, JobId)>,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpawningActivityJobsState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, DepositAccountId)>,
    // Vec for the same reason as AwaitingActivityCompletion::pending_jobs.
    pending_jobs: Vec<(DepositAccountId, JobId)>,
}

impl<Perms, E> DepositActivityProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<CoreCustomerAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<CustomerObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    /// Step 1: Query deposit accounts not escheatable (paginated) and spawn
    /// a per-account activity evaluation job for each. Transitions to
    /// AwaitingActivityCompletion when all pages are processed.
    async fn spawn_activity_jobs(
        &self,
        mut current_job: CurrentJob,
        mut state: SpawningActivityJobsState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            let mut op = current_job.begin_op().await?;
            let rows = self
                .deposits
                .list_account_ids_not_escheatable_in_op(&mut op, state.last_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    let job_id = JobId::new();
                    state.pending_jobs.push((*id, job_id));
                    JobSpec::new(
                        job_id,
                        EvaluateDepositAccountActivityConfig {
                            deposit_account_id: *id,
                            closing_time: self.config.closing_time,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            process_manager::spawn_in_op(&mut op, &self.evaluate_spawner, specs).await?;

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(
                    &mut op,
                    &DepositActivityState::SpawningActivityJobs(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        tracing::info!(
            entities = state.pending_jobs.len(),
            "Deposit activity spawning complete, transitioning to awaiting"
        );

        let new_state = DepositActivityState::AwaitingActivityCompletion {
            pending_jobs: state.pending_jobs,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    /// Step 2: Await completion of all spawned per-account jobs using
    /// Jobs.await_completion. This is a temporary fallback until a public
    /// DepositActivityEvaluated event is added — at that point, this should
    /// be replaced with outbox event streaming like the other two children.
    async fn await_activity_completion(
        &self,
        mut current_job: CurrentJob,
        pending_jobs: Vec<(DepositAccountId, JobId)>,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if pending_jobs.is_empty() {
            tracing::info!("No deposit accounts to track, completing immediately");
            return Ok(JobCompletion::Complete);
        }

        tracing::info!(
            remaining = pending_jobs.len(),
            "Awaiting deposit activity job completions"
        );

        let job_ids: Vec<_> = pending_jobs.iter().map(|(_, id)| *id).collect();
        let terminals =
            match process_manager::await_job_completions(&mut current_job, &self.jobs, &job_ids)
                .await?
            {
                Some(t) => t,
                None => return Ok(JobCompletion::RescheduleIn(Duration::ZERO)),
            };

        let failed = process_manager::failed_count(&terminals);
        if failed > 0 {
            return Err(format!(
                "{failed} of {} deposit activity child jobs did not complete successfully",
                terminals.len()
            )
            .into());
        }

        tracing::info!("All deposit activity evaluations completed");
        Ok(JobCompletion::Complete)
    }
}

#[async_trait]
impl<Perms, E> JobRunner for DepositActivityProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<CoreCustomerAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<CustomerObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.deposit-activity-process.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<DepositActivityState>()?
            .unwrap_or_default();

        match state {
            DepositActivityState::SpawningActivityJobs(spawning) => {
                self.spawn_activity_jobs(current_job, spawning).await
            }
            DepositActivityState::AwaitingActivityCompletion { pending_jobs } => {
                self.await_activity_completion(current_job, pending_jobs)
                    .await
            }
        }
    }
}
