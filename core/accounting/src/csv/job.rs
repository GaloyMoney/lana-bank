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
    type Initializer = GenerateAccountingCsvInitializer<Perms>;
}

pub struct GenerateAccountingCsvInitializer<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    repo: AccountingCsvRepo,
    storage: Storage,
    ledger_accounts: LedgerAccounts<Perms>,
    audit: Perms::Audit,
}

impl<Perms> GenerateAccountingCsvInitializer<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        repo: &AccountingCsvRepo,
        storage: &Storage,
        ledger_accounts: &LedgerAccounts<Perms>,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            repo: repo.clone(),
            storage: storage.clone(),
            ledger_accounts: ledger_accounts.clone(),
            audit: audit.clone(),
        }
    }
}

pub const GENERATE_ACCOUNTING_CSV_JOB: JobType = JobType::new("generate-accounting-csv");

impl<Perms> JobInitializer for GenerateAccountingCsvInitializer<Perms>
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
            audit: self.audit.clone(),
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
    audit: Perms::Audit,
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
        let mut export = self.repo.find_by_id(self.config.accounting_csv_id).await?;
        let mut db = self.repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_GENERATE,
            )
            .await?;

        let csv_type = export.csv_type;
        let csv_result = match csv_type {
            AccountingCsvType::LedgerAccount => {
                let ledger_account_id = export.ledger_account_id.ok_or_else(|| {
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
                let path_in_bucket = format!("accounting_csvs/{}.csv", export.id);
                let client = self.storage.client().await?;
                match client.upload(csv_data, &path_in_bucket, "text/csv").await {
                    Ok(_) => {
                        let _ = export
                            .file_uploaded(self.storage.bucket_name().to_string(), audit_info);
                    }
                    Err(e) => {
                        let _ = export.upload_failed(e.to_string(), audit_info);
                    }
                }
            }
            Err(e) => {
                let _ = export.upload_failed(e.to_string(), audit_info);
            }
        }

        self.repo.update_in_op(&mut db, &mut export).await?;
        let (now, tx) = (db.now(), db.into_tx());
        let db_static = es_entity::DbOp::new(tx, now);
        Ok(JobCompletion::CompleteWithOp(db_static))
    }
}
