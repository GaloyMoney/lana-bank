use serde::{Deserialize, Serialize};

use crate::{
    applicant::SumsubConfig, customer::CustomerConfig, job::JobExecutorConfig,
    ledger::LedgerConfig, loan::LoanConfig, report::ReportConfig, user::UserConfig,
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
    pub report: ReportConfig,
}
