use async_trait::async_trait;

use authz::PermissionCheck;

use crate::{
    CoreAccountingAction, CoreAccountingObject,
    ledger_account::LedgerAccounts,
    primitives::{AccountingCsvId, LedgerAccountId},
};

use audit::{AuditSvc, SystemSubject};
use chrono::Utc;
use cloud_storage::Storage;
use job::*;
use serde::{Deserialize, Serialize};

use super::{
    entity::{AccountingCsv, AccountingCsvType},
    error::AccountingCsvError,
    generate::GenerateCsv,
    repo::AccountingCsvRepo,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateAccountingCsvConfig<Perms> {
    pub accounting_csv_id: AccountingCsvId,
    pub _phantom: std::marker::PhantomData<Perms>,
}

impl<Perms> JobConfig for GenerateAccountingCsvConfig<Perms>
where
    Perms: authz::PermissionCheck + 'static,
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
    Perms: authz::PermissionCheck + 'static,
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
    Perms: authz::PermissionCheck + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        dbg!(&self.config.accounting_csv_id);
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

        let csv_type = export.csv_type();
        dbg!(&csv_type);
        let csv_result = match csv_type {
            AccountingCsvType::LedgerAccount => {
                let ledger_account_id = export.ledger_account_id().ok_or_else(|| {
                    AccountingCsvError::MissingRequiredField("ledger_account_id".to_string())
                })?;

                self.generator
                    .generate_ledger_account_csv(
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(),
                        ledger_account_id,
                    )
                    .await
            }
            AccountingCsvType::ProfitAndLoss => {
                // TODO:  profit and loss report generation
                Err(AccountingCsvError::UnsupportedCsvType)
            }
            AccountingCsvType::BalanceSheet => {
                // TODO:  balance sheet report generation
                Err(AccountingCsvError::UnsupportedCsvType)
            }
        };

        match csv_result {
            Ok(csv_data) => {
                let path_in_bucket = format!("accounting_csvs/{}.csv", export.id,);
                match self
                    .storage
                    .upload(csv_data, &path_in_bucket, "text/csv")
                    .await
                {
                    Ok(_) => {
                        export.file_uploaded(
                            path_in_bucket,
                            self.storage.bucket_name().to_string(),
                            audit_info,
                        );
                    }
                    Err(e) => {
                        export.upload_failed(e.to_string(), audit_info);
                    }
                }
            }
            Err(e) => {
                export.upload_failed(e.to_string(), audit_info);
            }
        }

        self.repo.update_in_op(&mut db, &mut export).await?;
        db.commit().await?;

        Ok(JobCompletion::Complete)
    }
}
