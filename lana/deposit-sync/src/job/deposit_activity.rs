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
use core_eod::deposit_activity_process::{
    DEPOSIT_ACTIVITY_PROCESS_JOB_TYPE, DepositActivityProcessConfig,
};
use governance::GovernanceEvent;
use job::{error::JobError, *};
use obix::out::OutboxEventMarker;

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
        jobs: &Jobs,
        deposits: &CoreDeposit<Perms, E>,
        evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
    ) -> Self {
        Self {
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
    jobs: Jobs,
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DepositActivityState {
    #[default]
    Collecting(DepositActivityCollectingState),
    Tracking {
        entity_job_ids: Vec<JobId>,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepositActivityCollectingState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, DepositAccountId)>,
    entity_job_ids: Vec<JobId>,
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
    async fn run_collecting(
        &self,
        mut current_job: CurrentJob,
        mut state: DepositActivityCollectingState,
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
                    let job_id = core_eod::eod_entity_id(
                        &self.config.date,
                        "deposit-activity",
                        &(*id).into(),
                    );
                    state.entity_job_ids.push(job_id);
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

            match self.evaluate_spawner.spawn_all_in_op(&mut op, specs).await {
                Ok(_) | Err(JobError::DuplicateId(_)) => {}
                Err(e) => return Err(e.into()),
            }

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(
                    &mut op,
                    &DepositActivityState::Collecting(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        tracing::info!(
            jobs_spawned = state.entity_job_ids.len(),
            "Deposit activity collection complete, transitioning to tracking"
        );

        let new_state = DepositActivityState::Tracking {
            entity_job_ids: state.entity_job_ids,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn run_tracking(
        &self,
        _current_job: CurrentJob,
        entity_job_ids: Vec<JobId>,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing::info!(
            total_jobs = entity_job_ids.len(),
            "Awaiting completion of per-entity deposit activity jobs"
        );

        let results = futures::future::try_join_all(
            entity_job_ids
                .iter()
                .map(|id| self.jobs.await_completion(*id)),
        )
        .await?;

        let failed: Vec<_> = entity_job_ids
            .iter()
            .zip(results.iter())
            .filter(|(_, state)| *state != &JobTerminalState::Completed)
            .map(|(id, state)| (*id, state.clone()))
            .collect();

        if !failed.is_empty() {
            tracing::error!(
                ?failed,
                "Some deposit activity entity jobs did not complete successfully"
            );
            return Err(format!("{} deposit activity entity jobs failed", failed.len()).into());
        }

        tracing::info!("All deposit activity entity jobs completed");
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
            DepositActivityState::Collecting(collecting) => {
                self.run_collecting(current_job, collecting).await
            }
            DepositActivityState::Tracking { entity_job_ids } => {
                self.run_tracking(current_job, entity_job_ids).await
            }
        }
    }
}
