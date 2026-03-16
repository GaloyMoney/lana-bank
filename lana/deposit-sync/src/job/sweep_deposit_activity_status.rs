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

const SWEEP_DEPOSIT_ACTIVITY_STATUS_JOB: JobType =
    JobType::new("command.deposit-sync.sweep-deposit-activity-status");
const PAGE_SIZE: usize = 100;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepDepositActivityStatusConfig {
    pub closing_time: chrono::DateTime<chrono::Utc>,
}

pub struct SweepDepositActivityStatusJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    deposits: CoreDeposit<Perms, E>,
    classify_spawner: ClassifyDepositAccountActivityJobSpawner,
}

impl<Perms, E> SweepDepositActivityStatusJobInit<Perms, E>
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

impl<Perms, E> JobInitializer for SweepDepositActivityStatusJobInit<Perms, E>
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
    type Config = SweepDepositActivityStatusConfig;

    fn job_type(&self) -> JobType {
        SWEEP_DEPOSIT_ACTIVITY_STATUS_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SweepDepositActivityStatusJobRunner {
            config: job.config()?,
            deposits: self.deposits.clone(),
            classify_spawner: self.classify_spawner.clone(),
        }))
    }
}

struct SweepDepositActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    config: SweepDepositActivityStatusConfig,
    deposits: CoreDeposit<Perms, E>,
    classify_spawner: ClassifyDepositAccountActivityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct SweepState {
    last_account_id: Option<DepositAccountId>,
}

#[async_trait]
impl<Perms, E> JobRunner for SweepDepositActivityStatusJobRunner<Perms, E>
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
        name = "deposit-sync.sweep-deposit-activity-status.process_command",
        skip_all
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<SweepState>()?
            .unwrap_or_default();

        let reclassifications = self
            .deposits
            .collect_activity_reclassifications(self.config.closing_time)
            .await?;

        let remaining: Vec<_> = reclassifications
            .into_iter()
            .filter(|(id, _)| state.last_account_id.is_none_or(|cursor| *id > cursor))
            .collect();

        for chunk in remaining.chunks(PAGE_SIZE) {
            let specs: Vec<_> = chunk
                .iter()
                .map(|(id, activity)| {
                    JobSpec::new(
                        JobId::new(),
                        ClassifyDepositAccountActivityConfig {
                            deposit_account_id: *id,
                            new_activity_status: *activity,
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

            state.last_account_id = chunk.last().map(|(id, _)| *id);
            current_job
                .update_execution_state_in_op(&mut op, &state)
                .await?;
            op.commit().await?;
        }

        Ok(JobCompletion::Complete)
    }
}

pub type SweepDepositActivityStatusJobSpawner = JobSpawner<SweepDepositActivityStatusConfig>;
