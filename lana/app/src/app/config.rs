use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::{
    access::config::AccessConfig, applicant::SumsubConfig, credit::CreditConfig,
    custody::CustodyConfig, customer_sync::CustomerSyncConfig, deposit_sync::DepositSyncConfig,
    job::JobsConfig, notification::NotificationConfig, report::ReportConfig,
    storage::config::StorageConfig, user_onboarding::UserOnboardingConfig,
};
use keycloak_admin::KeycloakConnectionConfig;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub job_execution: JobsConfig,
    #[serde(default)]
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
    pub deposit_sync: DepositSyncConfig,
    #[serde(default)]
    pub accounting_init: AccountingInitConfig,
    #[serde(default)]
    pub custody: CustodyConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub report: ReportConfig,
    #[serde(default)]
    pub keycloak: KeycloakConnectionConfig,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AccountingInitConfig {
    #[serde(default)]
    pub chart_of_accounts_seed_path: Option<PathBuf>,
    #[serde(default)]
    pub deposit_config_path: Option<PathBuf>,
    #[serde(default)]
    pub credit_config_path: Option<PathBuf>,
    #[serde(default)]
    pub balance_sheet_config_path: Option<PathBuf>,
    #[serde(default)]
    pub profit_and_loss_config_path: Option<PathBuf>,
}
