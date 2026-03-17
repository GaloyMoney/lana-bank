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
use core_eod::deposit_activity::{DEPOSIT_ACTIVITY_JOB_TYPE, DepositActivityConfig};
use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;

use super::evaluate_deposit_account_activity::{
    EvaluateDepositAccountActivityConfig, EvaluateDepositAccountActivityJobSpawner,
};

const PAGE_SIZE: i64 = 100;

pub struct DepositActivityJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

impl<Perms, E> DepositActivityJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(
        deposits: &CoreDeposit<Perms, E>,
        evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
    ) -> Self {
        Self {
            deposits: deposits.clone(),
            evaluate_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for DepositActivityJobInit<Perms, E>
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
    type Config = DepositActivityConfig;

    fn job_type(&self) -> JobType {
        DEPOSIT_ACTIVITY_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DepositActivityJobRunner {
            config: job.config()?,
            deposits: self.deposits.clone(),
            evaluate_spawner: self.evaluate_spawner.clone(),
        }))
    }
}

struct DepositActivityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    config: DepositActivityConfig,
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepositActivityCollectingState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, DepositAccountId)>,
    jobs_spawned: usize,
}

#[async_trait]
impl<Perms, E> JobRunner for DepositActivityJobRunner<Perms, E>
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
        name = "eod.deposit-activity.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<DepositActivityCollectingState>()?
            .unwrap_or_default();

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

            state.jobs_spawned += specs.len();
            self.evaluate_spawner
                .spawn_all_in_op(&mut op, specs)
                .await?;

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(&mut op, &state)
                .await?;
            op.commit().await?;
        }

        tracing::info!(
            jobs_spawned = state.jobs_spawned,
            "Deposit activity collection complete"
        );

        Ok(JobCompletion::Complete)
    }
}
