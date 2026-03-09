use std::{marker::PhantomData, time::Duration};

use async_trait::async_trait;
use audit::AuditSvc;
use authz::PermissionCheck;
use chrono::{DateTime, Utc};
use job::{
    CurrentJob, Job, JobCompletion, JobInitializer, JobRunner, JobSpawner, JobType, RetrySettings,
};
use obix::out::OutboxEventMarker;
use serde::{Deserialize, Serialize};
use tokio::select;

use crate::{
    CUSTODIAN_SYNC, CoreCustodyAction, CoreCustodyError, CoreCustodyEvent, CoreCustodyObject,
    CustodianRepo, CustodyConfig, WalletId, WalletRepo,
};

const SELF_CUSTODY_BALANCE_SYNC_INTERVAL: Duration = Duration::from_secs(60);
pub const SELF_CUSTODY_BALANCE_SYNC_JOB: JobType =
    JobType::new("cron.core-custody.self-custody-balance-sync");

#[derive(Clone, Serialize, Deserialize)]
pub struct SelfCustodyBalanceSyncJobConfig<E>
where
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    pub _phantom: PhantomData<E>,
}

pub struct SelfCustodyBalanceSyncJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    authz: Perms,
    custodians: CustodianRepo,
    wallets: WalletRepo<E>,
    encryption_config: encryption::EncryptionConfig,
    config: CustodyConfig,
}

impl<Perms, E> SelfCustodyBalanceSyncJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    pub fn new(
        authz: &Perms,
        custodians: &CustodianRepo,
        wallets: &WalletRepo<E>,
        encryption_config: &encryption::EncryptionConfig,
        config: &CustodyConfig,
    ) -> Self {
        Self {
            authz: authz.clone(),
            custodians: custodians.clone(),
            wallets: wallets.clone(),
            encryption_config: encryption_config.clone(),
            config: config.clone(),
        }
    }
}

impl<Perms, E> JobInitializer for SelfCustodyBalanceSyncJobInit<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    type Config = SelfCustodyBalanceSyncJobConfig<E>;

    fn job_type(&self) -> JobType {
        SELF_CUSTODY_BALANCE_SYNC_JOB
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SelfCustodyBalanceSyncJobRunner {
            authz: self.authz.clone(),
            custodians: self.custodians.clone(),
            wallets: self.wallets.clone(),
            encryption_config: self.encryption_config.clone(),
            config: self.config.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

struct SelfCustodyBalanceSyncJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    authz: Perms,
    custodians: CustodianRepo,
    wallets: WalletRepo<E>,
    encryption_config: encryption::EncryptionConfig,
    config: CustodyConfig,
}

#[async_trait]
impl<Perms, E> JobRunner for SelfCustodyBalanceSyncJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            self.sync_once(current_job.clock().now()).await?;

            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %SELF_CUSTODY_BALANCE_SYNC_JOB,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                _ = tokio::time::sleep(SELF_CUSTODY_BALANCE_SYNC_INTERVAL) => {}
            }
        }
    }
}

impl<Perms, E> SelfCustodyBalanceSyncJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    async fn sync_once(&self, update_time: DateTime<Utc>) -> Result<(), CoreCustodyError> {
        let mut custodian_op = self.custodians.begin_op().await?;
        let custodians = self.custodians.list_all_in_op(&mut custodian_op).await?;
        custodian_op.commit().await?;

        for custodian in custodians {
            if !custodian.requires_balance_polling() {
                continue;
            }

            let client = custodian.clone().custodian_client(
                &self.encryption_config.encryption_key,
                &self.config.custody_providers,
            )?;

            let mut wallet_op = self.wallets.begin_op().await?;
            let wallets = self
                .wallets
                .list_all_by_custodian_id_in_op(&mut wallet_op, custodian.id)
                .await?;
            wallet_op.commit().await?;

            for wallet in wallets {
                let new_balance = client
                    .fetch_wallet_balance(&wallet.external_wallet_id, &wallet.address)
                    .await?;

                if let Some(new_balance) = new_balance {
                    self.record_balance(wallet.id, new_balance, update_time)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn record_balance(
        &self,
        wallet_id: WalletId,
        new_balance: money::Satoshis,
        update_time: DateTime<Utc>,
    ) -> Result<(), CoreCustodyError> {
        let mut op = self.wallets.begin_op().await?;
        let mut wallet = self.wallets.find_by_id_in_op(&mut op, wallet_id).await?;

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
                CUSTODIAN_SYNC,
                CoreCustodyObject::wallet(wallet.id),
                CoreCustodyAction::WALLET_UPDATE,
            )
            .await?;

        if wallet
            .update_balance(new_balance, update_time)
            .did_execute()
        {
            self.wallets.update_in_op(&mut op, &mut wallet).await?;
        }

        op.commit().await?;

        Ok(())
    }
}
