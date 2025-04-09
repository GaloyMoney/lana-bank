use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::audit::AuditInfo;
use crate::ledger_account::LedgerAccounts;
use crate::primitives::{AccountingCsvId, LedgerAccountId};
use crate::storage::Storage;
use job::{Job, JobCompletion, JobConfig, JobCurrentJob, JobInitializer, JobRunner, JobType};

use super::error::AccountingCsvError;
use super::generate::GenerateCsv;
use super::repo::AccountingCsvRepo;

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateAccountingCsvConfig {
    pub accounting_csv_id: AccountingCsvId,
}

impl JobConfig for GenerateAccountingCsvConfig {
    type Initializer = GenerateAccountingCsvInitializer;
}

pub struct GenerateAccountingCsvInitializer {
    repo: AccountingCsvRepo,
    storage: Storage,
    ledger_accounts: LedgerAccounts,
}

impl GenerateAccountingCsvInitializer {
    pub fn new(
        repo: &AccountingCsvRepo,
        storage: &Storage,
        ledger_accounts: &LedgerAccounts,
    ) -> Self {
        Self {
            repo: repo.clone(),
            storage: storage.clone(),
            ledger_accounts: ledger_accounts.clone(),
        }
    }
}

pub const GENERATE_ACCOUNTING_CSV_JOB: JobType = JobType::new("generate-accounting-csv");
impl JobInitializer for GenerateAccountingCsvInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        GENERATE_ACCOUNTING_CSV_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(GenerateAccountingCsvJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            storage: self.storage.clone(),
            generator: GenerateCsv::new(&self.ledger_accounts),
        }))
    }
}

pub struct GenerateAccountingCsvJobRunner {
    config: GenerateAccountingCsvConfig,
    repo: AccountingCsvRepo,
    storage: Storage,
    generator: GenerateCsv,
}

#[async_trait]
impl JobRunner for GenerateAccountingCsvJobRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut export = self.repo.find_by_id(self.config.accounting_csv_id).await?;
        let mut db = self.repo.begin_op().await?;

        let csv_type = export.csv_type();
        let csv_result = match &csv_type {
            AccountingCsvType::LedgerAccount { ledger_account_id } => {
                self.generator
                    .generate_ledger_account_csv(*ledger_account_id)
                    .await
            }
        };

        // TODO upload to storage

        self.repo.update_in_op(&mut db, &mut export).await?;
        db.commit().await?;

        Ok(JobCompletion::Complete)
    }
}
