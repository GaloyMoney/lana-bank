use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountId,
    GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;

use super::evaluate_deposit_account_activity::{
    EvaluateDepositAccountActivityConfig, EvaluateDepositAccountActivityJobSpawner,
};

const COLLECT_ACCOUNTS_FOR_ACTIVITY_EVALUATION_JOB: JobType =
    JobType::new("task.collect-accounts-for-activity-evaluation");
const PAGE_SIZE: i64 = 100;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectAccountsForActivityEvaluationConfig {
    pub closing_time: chrono::DateTime<chrono::Utc>,
}

pub struct CollectAccountsForActivityEvaluationJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

impl<Perms, E> CollectAccountsForActivityEvaluationJobInit<Perms, E>
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

impl<Perms, E> JobInitializer for CollectAccountsForActivityEvaluationJobInit<Perms, E>
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
    type Config = CollectAccountsForActivityEvaluationConfig;

    fn job_type(&self) -> JobType {
        COLLECT_ACCOUNTS_FOR_ACTIVITY_EVALUATION_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CollectAccountsForActivityEvaluationJobRunner {
            config: job.config()?,
            deposits: self.deposits.clone(),
            evaluate_spawner: self.evaluate_spawner.clone(),
        }))
    }
}

struct CollectAccountsForActivityEvaluationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    config: CollectAccountsForActivityEvaluationConfig,
    deposits: CoreDeposit<Perms, E>,
    evaluate_spawner: EvaluateDepositAccountActivityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CollectAccountsForActivityEvaluationState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, DepositAccountId)>,
}

#[async_trait]
impl<Perms, E> JobRunner for CollectAccountsForActivityEvaluationJobRunner<Perms, E>
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
    #[instrument(
        name = "deposit-sync.collect-accounts-for-activity-evaluation.run",
        skip(self, current_job),
        fields(closing_time = %self.config.closing_time)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CollectAccountsForActivityEvaluationState>()?
            .unwrap_or_default();

        loop {
            let rows = self
                .deposits
                .list_account_ids_for_activity_evaluation(state.last_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    JobSpec::new(
                        JobId::new(),
                        EvaluateDepositAccountActivityConfig {
                            deposit_account_id: *id,
                            closing_time: self.config.closing_time,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            let mut op = current_job.begin_op().await?;
            self.evaluate_spawner
                .spawn_all_in_op(&mut op, specs)
                .await?;

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(&mut op, &state)
                .await?;
            op.commit().await?;
        }

        Ok(JobCompletion::Complete)
    }
}

pub type CollectAccountsForActivityEvaluationJobSpawner =
    JobSpawner<CollectAccountsForActivityEvaluationConfig>;
