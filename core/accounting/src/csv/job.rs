use async_trait::async_trait;

use authz::PermissionCheck;

use audit::AuditSvc;
use cloud_storage::Storage;
use job::*;
use serde::{Deserialize, Serialize};

use crate::{ledger_account::LedgerAccounts, primitives::AccountingCsvId};

use super::{
    CoreAccountingAction, CoreAccountingObject, error::AccountingCsvError, generate::GenerateCsv,
    primitives::AccountingCsvType, repo::AccountingCsvRepo,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateAccountingCsvConfig<Perms> {
    pub accounting_csv_id: AccountingCsvId,
    pub _phantom: std::marker::PhantomData<Perms>,
}

impl<Perms> JobConfig for GenerateAccountingCsvConfig<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    type Initializer = GenerateAccountingCsvInit<Perms>;
}

pub struct GenerateAccountingCsvInit<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    repo: AccountingCsvRepo,
    storage: Storage,
    ledger_accounts: LedgerAccounts<Perms>,
}

impl<Perms> GenerateAccountingCsvInit<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        repo: &AccountingCsvRepo,
        storage: &Storage,
        ledger_accounts: &LedgerAccounts<Perms>,
    ) -> Self {
        Self {
            repo: repo.clone(),
            storage: storage.clone(),
            ledger_accounts: ledger_accounts.clone(),
        }
    }
}

pub const GENERATE_ACCOUNTING_CSV_JOB: JobType = JobType::new("generate-accounting-csv");

impl<Perms> JobInitializer for GenerateAccountingCsvInit<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
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

pub struct GenerateAccountingCsvJobRunner<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    config: GenerateAccountingCsvConfig<Perms>,
    repo: AccountingCsvRepo,
    storage: Storage,
    generator: GenerateCsv<Perms>,
}

#[async_trait]
impl<Perms> JobRunner for GenerateAccountingCsvJobRunner<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut csv_job = self.repo.find_by_id(self.config.accounting_csv_id).await?;
        let mut db = self.repo.begin_op().await?;

        let csv_type = csv_job.csv_type;
        let csv_result = match csv_type {
            AccountingCsvType::LedgerAccount => {
                let ledger_account_id = csv_job.ledger_account_id.ok_or_else(|| {
                    AccountingCsvError::MissingRequiredField("ledger_account_id".to_string())
                })?;

                self.generator
                    .generate_ledger_account_csv(ledger_account_id)
                    .await
            }
            AccountingCsvType::ProfitAndLoss => Err(AccountingCsvError::UnsupportedCsvType),
            AccountingCsvType::BalanceSheet => Err(AccountingCsvError::UnsupportedCsvType),
        };

        match csv_result {
            Ok(csv_data) => {
                match self
                    .storage
                    .upload(csv_data, &csv_job.path_in_storage, "text/csv")
                    .await
                {
                    Ok(_) => {
                        let _ = csv_job.file_uploaded(self.storage.bucket_name().to_string());
                    }
                    Err(e) => {
                        let _ = csv_job.upload_failed(e.to_string());
                    }
                }
            }
            Err(e) => {
                let _ = csv_job.upload_failed(e.to_string());
            }
        }

        self.repo.update_in_op(&mut db, &mut csv_job).await?;
        let (now, tx) = (db.now(), db.into_tx());
        let db_static = es_entity::DbOp::new(tx, now);
        Ok(JobCompletion::CompleteWithOp(db_static))
    }
}
