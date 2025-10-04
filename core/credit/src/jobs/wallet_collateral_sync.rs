use async_trait::async_trait;
use futures::StreamExt;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{Outbox, OutboxEventMarker};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{Collaterals, CoreCreditAction, CoreCreditEvent, CoreCreditObject};

#[derive(serde::Serialize)]
pub(crate) struct PermanentWalletCollateralSyncJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> PermanentWalletCollateralSyncJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for PermanentWalletCollateralSyncJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for PermanentWalletCollateralSyncJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = PermanentWalletCollateralSyncInit<Perms, E>;
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct PermanentWalletCollateralSyncJobData {
    sequence: outbox::EventSequence,
}

pub struct PermanentWalletCollateralSyncJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    collaterals: Collaterals<Perms, E>,
    outbox: Outbox<E>,
}

#[async_trait]
impl<Perms, E> JobRunner for PermanentWalletCollateralSyncJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PermanentWalletCollateralSyncJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCustodyEvent::WalletBalanceChanged {
                id,
                new_balance,
                changed_at,
            }) = message.as_ref().as_event()
            {
                self.collaterals
                    .record_collateral_update_via_custodian_sync(
                        *id,
                        *new_balance,
                        changed_at.date_naive(),
                    )
                    .await?;

                state.sequence = message.sequence;
                current_job.update_execution_state(state).await?;
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

pub struct PermanentWalletCollateralSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    collaterals: Collaterals<Perms, E>,
}

impl<Perms, E> PermanentWalletCollateralSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(outbox: &Outbox<E>, collaterals: &Collaterals<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            collaterals: collaterals.clone(),
        }
    }
}

const WALLET_COLLATERAL_SYNC_JOB: JobType = JobType::new("permanent-wallet-collateral-sync");
impl<Perms, E> JobInitializer for PermanentWalletCollateralSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        WALLET_COLLATERAL_SYNC_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PermanentWalletCollateralSyncJobRunner {
            outbox: self.outbox.clone(),
            collaterals: self.collaterals.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}
