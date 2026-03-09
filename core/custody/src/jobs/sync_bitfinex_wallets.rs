use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{select, time::Duration};

use job::*;

use crate::custodian::{CustodianConfig, CustodianRepo, CustodyProviderConfig};

use encryption::EncryptionConfig;

const SYNC_INTERVAL: Duration = Duration::from_secs(30);
const SYNC_BITFINEX_WALLETS_JOB_TYPE: JobType = JobType::new("cron.custody.sync-bitfinex-wallets");

#[derive(Clone, Serialize, Deserialize)]
pub struct SyncBitfinexWalletsJobConfig;

pub struct SyncBitfinexWalletsJobInit {
    custodians: CustodianRepo,
    encryption_config: EncryptionConfig,
    custody_providers: CustodyProviderConfig,
}

impl SyncBitfinexWalletsJobInit {
    pub fn new(
        custodians: CustodianRepo,
        encryption_config: EncryptionConfig,
        custody_providers: CustodyProviderConfig,
    ) -> Self {
        Self {
            custodians,
            encryption_config,
            custody_providers,
        }
    }
}

impl JobInitializer for SyncBitfinexWalletsJobInit {
    type Config = SyncBitfinexWalletsJobConfig;

    fn job_type(&self) -> JobType {
        SYNC_BITFINEX_WALLETS_JOB_TYPE
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SyncBitfinexWalletsJobRunner {
            custodians: self.custodians.clone(),
            encryption_config: self.encryption_config.clone(),
            custody_providers: self.custody_providers.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct SyncBitfinexWalletsJobRunner {
    custodians: CustodianRepo,
    encryption_config: EncryptionConfig,
    custody_providers: CustodyProviderConfig,
}

#[async_trait]
impl JobRunner for SyncBitfinexWalletsJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            if let Err(e) = self.sync_wallets().await {
                tracing::warn!(
                    error = %e,
                    "Failed to sync Bitfinex wallets"
                );
            }

            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %SYNC_BITFINEX_WALLETS_JOB_TYPE,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                _ = tokio::time::sleep(SYNC_INTERVAL) => {
                    tracing::debug!(job_id = %current_job.id(), "Sleep completed, continuing");
                }
            }
        }
    }
}

impl SyncBitfinexWalletsJobRunner {
    #[tracing::instrument(name = "custody.sync_bitfinex_wallets", skip(self))]
    async fn sync_wallets(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let custodian = match self
            .custodians
            .find_by_provider("bitfinex".to_string())
            .await
        {
            Ok(c) => c,
            Err(_) => {
                tracing::debug!("No Bitfinex custodian configured, skipping sync");
                return Ok(());
            }
        };

        let config = custodian.custodian_config(&self.encryption_config.encryption_key)?;

        let bitfinex_config = match config {
            CustodianConfig::Bitfinex(c) => c,
            _ => {
                tracing::warn!("Custodian provider is 'bitfinex' but config is not Bitfinex");
                return Ok(());
            }
        };

        let client = bitfinex::BitfinexClient::try_new(
            bitfinex_config.into(),
            self.custody_providers.bitfinex_directory.clone(),
        )?;

        let wallets = client.list_wallets().await?;

        for wallet in &wallets {
            if wallet.currency == "BTC" && wallet.wallet_type == "exchange" {
                tracing::info!(
                    balance = wallet.balance,
                    available = wallet.available_balance,
                    "Bitfinex BTC exchange wallet balance"
                );
            }
        }

        Ok(())
    }
}
