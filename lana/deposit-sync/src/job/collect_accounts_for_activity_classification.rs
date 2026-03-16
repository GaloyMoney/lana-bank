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

use super::classify_deposit_account_activity::{
    ClassifyDepositAccountActivityConfig, ClassifyDepositAccountActivityJobSpawner,
};

const COLLECT_ACCOUNTS_FOR_ACTIVITY_CLASSIFICATION_JOB: JobType =
    JobType::new("command.deposit-sync.collect-accounts-for-activity-classification");
const PAGE_SIZE: i64 = 100;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectAccountsForActivityClassificationConfig {
    pub closing_time: chrono::DateTime<chrono::Utc>,
}

pub struct CollectAccountsForActivityClassificationJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    deposits: CoreDeposit<Perms, E>,
    classify_spawner: ClassifyDepositAccountActivityJobSpawner,
}

impl<Perms, E> CollectAccountsForActivityClassificationJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(
        deposits: &CoreDeposit<Perms, E>,
        classify_spawner: ClassifyDepositAccountActivityJobSpawner,
    ) -> Self {
        Self {
            deposits: deposits.clone(),
            classify_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for CollectAccountsForActivityClassificationJobInit<Perms, E>
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
    type Config = CollectAccountsForActivityClassificationConfig;

    fn job_type(&self) -> JobType {
        COLLECT_ACCOUNTS_FOR_ACTIVITY_CLASSIFICATION_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            CollectAccountsForActivityClassificationJobRunner {
                config: job.config()?,
                deposits: self.deposits.clone(),
                classify_spawner: self.classify_spawner.clone(),
            },
        ))
    }
}

struct CollectAccountsForActivityClassificationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    config: CollectAccountsForActivityClassificationConfig,
    deposits: CoreDeposit<Perms, E>,
    classify_spawner: ClassifyDepositAccountActivityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CollectAccountsForActivityClassificationState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, DepositAccountId)>,
}

#[async_trait]
impl<Perms, E> JobRunner for CollectAccountsForActivityClassificationJobRunner<Perms, E>
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
        name = "deposit-sync.collect-accounts-for-activity-classification.process_command",
        skip(self, current_job),
        fields(closing_time = %self.config.closing_time)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CollectAccountsForActivityClassificationState>()?
            .unwrap_or_default();

        loop {
            let rows = self
                .deposits
                .list_account_ids_for_activity_classification(state.last_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    JobSpec::new(
                        JobId::new(),
                        ClassifyDepositAccountActivityConfig {
                            deposit_account_id: *id,
                            closing_time: self.config.closing_time,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            let mut op = current_job.begin_op().await?;
            self.classify_spawner
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

pub type CollectAccountsForActivityClassificationJobSpawner =
    JobSpawner<CollectAccountsForActivityClassificationConfig>;
