use std::collections::HashSet;

use async_trait::async_trait;
use futures::StreamExt;
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
        deposits: &CoreDeposit<Perms, E>,
        evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
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
            outbox: self.outbox.clone(),
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
    outbox: Outbox<E>,
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DepositActivityState {
    #[default]
    Collecting(DepositActivityCollectingState),
    Tracking {
        pending: HashSet<DepositAccountId>,
        start_sequence: i64,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepositActivityCollectingState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, DepositAccountId)>,
    pending: HashSet<DepositAccountId>,
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
                    state.pending.insert(*id);
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

        let start_sequence = self.outbox.current_sequence().await?;

        tracing::info!(
            entities = state.pending.len(),
            start_sequence,
            "Deposit activity collection complete, transitioning to tracking"
        );

        let new_state = DepositActivityState::Tracking {
            pending: state.pending,
            start_sequence,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn run_tracking(
        &self,
        mut current_job: CurrentJob,
        mut pending: HashSet<DepositAccountId>,
        mut start_sequence: i64,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if pending.is_empty() {
            tracing::info!("No deposit accounts to track, completing immediately");
            return Ok(JobCompletion::Complete);
        }

        tracing::info!(
            remaining = pending.len(),
            start_sequence,
            "Streaming outbox events for deposit activity completion"
        );

        let mut stream = self.outbox.listen_persisted(Some(start_sequence));

        loop {
            tokio::select! {
                Some(event) = stream.next() => {
                    // TODO: No public deposit-activity-evaluated event exists yet.
                    // When added, match it here and remove from pending set.
                    // For now, consume events to maintain the streaming structure.
                    if let Some(payload) = event.payload.as_ref() {
                        if let Some(_deposit_event) = payload.as_event::<CoreDepositEvent>() {
                            // Future: match on deposit activity completion event
                            // and extract account ID to remove from pending set
                        }
                    }
                    start_sequence = event.sequence;

                    if pending.is_empty() {
                        tracing::info!("All deposit activity evaluations completed");
                        return Ok(JobCompletion::Complete);
                    }
                }
                _ = current_job.shutdown_requested() => {
                    let state = DepositActivityState::Tracking {
                        pending,
                        start_sequence,
                    };
                    current_job.update_execution_state(&state).await?;
                    tracing::info!("Shutdown requested, rescheduling deposit activity tracking");
                    return Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO));
                }
            }
        }
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
            DepositActivityState::Tracking {
                pending,
                start_sequence,
            } => {
                self.run_tracking(current_job, pending, start_sequence)
                    .await
            }
        }
    }
}
