use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{select, time::Duration};

use job::*;

use audit::AuditSvc;
use authz::PermissionCheck;
use encryption::EncryptionConfig;
use obix::out::{Outbox, OutboxEventMarker};
use sqlx::PgPool;

use crate::{
    CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject, CustodyConfig, CustodyPublisher,
    custodian::{CustodianConfig, CustodianRepo, SelfCustodyClient},
    wallet::WalletRepo,
};

const POLL_INTERVAL: Duration = Duration::from_secs(60);
pub const POLL_SELF_CUSTODY_BALANCES_JOB_TYPE: JobType =
    JobType::new("cron.custody.poll-self-custody-balances");

#[derive(Clone, Serialize, Deserialize)]
pub struct PollSelfCustodyBalancesJobConfig<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

pub struct PollSelfCustodyBalancesJobInit<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    custodians: CustodianRepo,
    wallets: WalletRepo<E>,
    encryption_config: EncryptionConfig,
    config: CustodyConfig,
    authz: Perms,
}

impl<Perms, E> PollSelfCustodyBalancesJobInit<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    pub fn new(
        pool: &PgPool,
        authz: &Perms,
        encryption_config: &EncryptionConfig,
        config: &CustodyConfig,
        outbox: &Outbox<E>,
        clock: es_entity::clock::ClockHandle,
    ) -> Self {
        Self {
            custodians: CustodianRepo::new(pool, clock.clone()),
            wallets: WalletRepo::new(pool, &CustodyPublisher::new(outbox), clock),
            encryption_config: encryption_config.clone(),
            config: config.clone(),
            authz: authz.clone(),
        }
    }
}

impl<Perms, E> JobInitializer for PollSelfCustodyBalancesJobInit<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    type Config = PollSelfCustodyBalancesJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        POLL_SELF_CUSTODY_BALANCES_JOB_TYPE
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PollSelfCustodyBalancesJobRunner {
            custodians: self.custodians.clone(),
            wallets: self.wallets.clone(),
            encryption_config: self.encryption_config.clone(),
            config: self.config.clone(),
            authz: self.authz.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

struct PollSelfCustodyBalancesJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    custodians: CustodianRepo,
    wallets: WalletRepo<E>,
    encryption_config: EncryptionConfig,
    config: CustodyConfig,
    authz: Perms,
}

#[async_trait]
impl<Perms, E> JobRunner for PollSelfCustodyBalancesJobRunner<Perms, E>
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
            if let Err(e) = self.poll_balances(&current_job).await {
                tracing::warn!(
                    error = ?e,
                    "Failed to poll self-custody balances, will retry next interval"
                );
            }

            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %POLL_SELF_CUSTODY_BALANCES_JOB_TYPE,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                _ = tokio::time::sleep(POLL_INTERVAL) => {
                    tracing::debug!(job_id = %current_job.id(), "Sleep completed, continuing");
                }
            }
        }
    }
}

impl<Perms, E> PollSelfCustodyBalancesJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(name = "custody.poll_self_custody_balances", skip_all)]
    async fn poll_balances(
        &self,
        current_job: &CurrentJob,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let custodian = match self
            .custodians
            .find_by_provider("self-custody".to_string())
            .await
        {
            Ok(c) => c,
            Err(_) => {
                tracing::debug!("No self-custody custodian found, skipping poll");
                return Ok(());
            }
        };

        let config = custodian.custodian_config(&self.encryption_config.encryption_key)?;

        let sc_config = match config {
            CustodianConfig::SelfCustody(sc) => sc,
            _ => return Ok(()),
        };

        let client = SelfCustodyClient::try_new(
            &sc_config,
            &self.config.custody_providers.self_custody_directory,
        )?;

        let all_wallets = self
            .wallets
            .list_by_id(Default::default(), es_entity::ListDirection::Ascending)
            .await?;

        let now = current_job.clock().now();

        for wallet in all_wallets
            .entities
            .iter()
            .filter(|w| w.custodian_id == custodian.id)
        {
            match client.get_address_balance(&wallet.address).await {
                Ok(balance_sats) => {
                    let new_balance = money::Satoshis::from(balance_sats);

                    let mut db = self.wallets.begin_op().await?;

                    let mut wallet = self
                        .wallets
                        .find_by_external_wallet_id_in_op(
                            &mut db,
                            wallet.external_wallet_id.clone(),
                        )
                        .await?;

                    self.authz
                        .audit()
                        .record_system_entry_in_op(
                            &mut db,
                            audit::SystemActor::from("self-custody-poller".to_string()),
                            CoreCustodyObject::wallet(wallet.id),
                            CoreCustodyAction::WALLET_UPDATE,
                        )
                        .await?;

                    if wallet.update_balance(new_balance, now).did_execute() {
                        self.wallets.update_in_op(&mut db, &mut wallet).await?;
                    }

                    db.commit().await?;
                }
                Err(e) => {
                    tracing::warn!(
                        address = %wallet.address,
                        error = ?e,
                        "Failed to fetch balance for self-custody wallet"
                    );
                }
            }
        }

        Ok(())
    }
}
