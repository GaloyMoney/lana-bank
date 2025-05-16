use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::{
    applicant::SumsubConfig, credit::CreditConfig, customer_sync::CustomerSyncConfig,
    deposit::DepositConfig, job::JobExecutorConfig, report::ReportConfig,
    service_account::ServiceAccountConfig, storage::config::StorageConfig,
    user_onboarding::UserOnboardingConfig,
};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub job_execution: JobExecutorConfig,
    #[serde(default)]
    pub sumsub: SumsubConfig,
    #[serde(default)]
    pub user: UserConfig,
    #[serde(default)]
    pub credit: CreditConfig,
    #[serde(default)]
    pub deposit: DepositConfig,
    #[serde(default)]
    pub service_account: ServiceAccountConfig,
    #[serde(default)]
    pub report: ReportConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub user_onboarding: UserOnboardingConfig,
    #[serde(default)]
    pub customer_sync: CustomerSyncConfig,
    #[serde(default)]
    pub chart_of_accounts_seed_path: Option<PathBuf>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct UserConfig {
    #[serde(default)]
    pub superuser_email: Option<String>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct ChartOfAccountsSeedPathsConfig {
    #[serde(default)]
    pub chart_of_accounts_seed_path: Option<PathBuf>,
    #[serde(default)]
    pub deposit_config_path: Option<PathBuf>,
    #[serde(default)]
    pub credit_config_path: Option<PathBuf>,
}

impl From<AppConfig> for ChartOfAccountsSeedPathsConfig {
    fn from(config: AppConfig) -> Self {
        Self {
            chart_of_accounts_seed_path: config.chart_of_accounts_seed_path,
            credit_config_path: config.credit.chart_of_accounts_config_path,
            deposit_config_path: config.deposit.chart_of_accounts_config_path,
        }
    }
}
