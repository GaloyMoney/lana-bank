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
