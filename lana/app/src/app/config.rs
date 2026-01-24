use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::{
    access::config::AccessConfig, credit::CreditConfig, custody::CustodyConfig,
    customer_sync::CustomerSyncConfig, deposit::DepositConfig, gotenberg::GotenbergConfig,
    job::JobPollerConfig, kyc::SumsubConfig, notification::NotificationConfig,
    report::ReportConfig, storage::config::StorageConfig, user_onboarding::UserOnboardingConfig,
};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub job_poller: JobPollerConfig,
    #[serde(default)]
    #[serde(skip)]
    pub sumsub: SumsubConfig,
    #[serde(default)]
    pub access: AccessConfig,
    #[serde(default)]
    pub credit: CreditConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub user_onboarding: UserOnboardingConfig,
    #[serde(default)]
    pub customer_sync: CustomerSyncConfig,
    #[serde(default)]
    pub accounting_init: AccountingInitConfig,
    #[serde(default)]
    pub custody: CustodyConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub report: ReportConfig,
    #[serde(default)]
    pub deposit: DepositConfig,
    #[serde(default)]
    pub gotenberg: GotenbergConfig,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountingInitConfig {
    pub chart_of_accounts_opening_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub chart_of_accounts_seed_path: Option<PathBuf>,
    #[serde(default)]
    pub deposit_config_path: Option<PathBuf>,
    #[serde(default)]
    pub credit_config_path: Option<PathBuf>,
    #[serde(default)]
    pub chart_of_accounts_integration_config_path: Option<PathBuf>,
}
