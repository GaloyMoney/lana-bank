use serde::{Deserialize, Serialize};

use crate::{
    applicant::SumsubConfig, credit_facility::CreditFacilityConfig, customer::CustomerConfig,
    job::JobExecutorConfig, ledger::LedgerConfig, loan::LoanConfig, report::ReportConfig,
    service_account::ServiceAccountConfig, storage::config::StorageConfig, user::UserConfig,
};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub job_execution: JobExecutorConfig,
    #[serde(default)]
    pub ledger: LedgerConfig,
    #[serde(default)]
    pub sumsub: SumsubConfig,
    #[serde(default)]
    pub user: UserConfig,
    #[serde(default)]
    pub customer: CustomerConfig,
    #[serde(default)]
    pub loan: LoanConfig,
    #[serde(default)]
    pub credit_facility: CreditFacilityConfig,
    #[serde(default)]
    pub service_account: ServiceAccountConfig,
    #[serde(default)]
    pub report: ReportConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}
