use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use core_deposit::{
    Activity, CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject,
    DepositAccountId, GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;

const CLASSIFY_DEPOSIT_ACCOUNT_ACTIVITY_JOB: JobType =
    JobType::new("command.deposit-sync.classify-deposit-account-activity");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassifyDepositAccountActivityConfig {
    pub deposit_account_id: DepositAccountId,
    pub new_activity_status: Activity,
}

pub struct ClassifyDepositAccountActivityJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    deposits: CoreDeposit<Perms, E>,
}

impl<Perms, E> ClassifyDepositAccountActivityJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(deposits: &CoreDeposit<Perms, E>) -> Self {
        Self {
            deposits: deposits.clone(),
        }
    }
}

impl<Perms, E> JobInitializer for ClassifyDepositAccountActivityJobInit<Perms, E>
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
    type Config = ClassifyDepositAccountActivityConfig;

    fn job_type(&self) -> JobType {
        CLASSIFY_DEPOSIT_ACCOUNT_ACTIVITY_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ClassifyDepositAccountActivityJobRunner {
            config: job.config()?,
            deposits: self.deposits.clone(),
        }))
    }
}

struct ClassifyDepositAccountActivityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    config: ClassifyDepositAccountActivityConfig,
    deposits: CoreDeposit<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ClassifyDepositAccountActivityJobRunner<Perms, E>
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
        name = "deposit-sync.classify-deposit-account-activity.process_command",
        skip(self, current_job),
        fields(deposit_account_id = %self.config.deposit_account_id)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        self.deposits
            .update_account_activity_in_op(
                &mut op,
                self.config.deposit_account_id,
                self.config.new_activity_status,
            )
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub type ClassifyDepositAccountActivityJobSpawner =
    JobSpawner<ClassifyDepositAccountActivityConfig>;
