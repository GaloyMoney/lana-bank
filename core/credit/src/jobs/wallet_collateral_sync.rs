use async_trait::async_trait;
use futures::StreamExt;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{Outbox, OutboxEventMarker};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{Collaterals, CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilities};

#[derive(serde::Serialize)]
pub struct WalletCollateralSyncJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> WalletCollateralSyncJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for WalletCollateralSyncJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for WalletCollateralSyncJobConfig<Perms, E>
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
    type Initializer = WalletCollateralSyncInit<Perms, E>;
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct WalletCollateralSyncJobData {
    sequence: outbox::EventSequence,
}

pub struct WalletCollateralSyncJobRunner<Perms, E>
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
    facilities: CreditFacilities<Perms, E>,
    collaterals: Collaterals<Perms, E>,
    outbox: Outbox<E>,
}

#[async_trait]
impl<Perms, E> JobRunner for WalletCollateralSyncJobRunner<Perms, E>
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
            .execution_state::<WalletCollateralSyncJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCustodyEvent::WalletBalanceChanged { id, new_balance }) =
                message.as_ref().as_event()
            {
                let credit_facility = self.facilities.find_by_custody_wallet(*id).await?;

                let effective = crate::time::now().date_naive();
                self.collaterals
                    .record_collateral_update_via_custodian_sync(
                        &credit_facility,
                        *new_balance,
                        effective,
                    )
                    .await?;

                state.sequence = message.sequence;
                current_job.update_execution_state(state).await?;
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

pub struct WalletCollateralSyncInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    facilities: CreditFacilities<Perms, E>,
    collaterals: Collaterals<Perms, E>,
}

impl<Perms, E> WalletCollateralSyncInit<Perms, E>
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
    pub fn new(
        outbox: &Outbox<E>,
        facilities: &CreditFacilities<Perms, E>,
        collaterals: &Collaterals<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            facilities: facilities.clone(),
            collaterals: collaterals.clone(),
        }
    }
}

const WALLET_COLLATERAL_SYNC_JOB: JobType = JobType::new("wallet-collateral-sync");
impl<Perms, E> JobInitializer for WalletCollateralSyncInit<Perms, E>
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
        Ok(Box::new(WalletCollateralSyncJobRunner {
            outbox: self.outbox.clone(),
            facilities: self.facilities.clone(),
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
