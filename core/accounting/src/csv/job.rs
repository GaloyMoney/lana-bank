use async_trait::async_trait;

use authz::PermissionCheck;

use audit::AuditSvc;
use cloud_storage::Storage;
use job::*;
use serde::{Deserialize, Serialize};

use crate::{ledger_account::LedgerAccounts, primitives::AccountingCsvId};

use super::{
    CoreAccountingAction, CoreAccountingObject, error::AccountingCsvError,
    primitives::AccountingCsvType, repo::AccountingCsvRepo,
};

use csv::Writer;
use rust_decimal::Decimal;

use cala_ledger::DebitOrCredit;

use crate::primitives::LedgerAccountId;

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
    audit: Perms::Audit,
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
        let mut csv = self.repo.find_by_id(self.config.accounting_csv_id).await?;
        let mut db = self.repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_GENERATE,
            )
            .await?;

        let csv_type = csv.csv_type;
        let csv_result = match csv_type {
            AccountingCsvType::LedgerAccount => {
                let ledger_account_id = csv.ledger_account_id.ok_or_else(|| {
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
                    .upload(csv_data, &csv.path_in_storage, "text/csv")
                    .await
                {
                    Ok(_) => {
                        let _ =
                            csv.file_uploaded(self.storage.bucket_name().to_string(), audit_info);
                    }
                    Err(e) => {
                        let _ = csv.upload_failed(e.to_string(), audit_info);
                    }
                }
            }
            Err(e) => {
                let _ = csv.upload_failed(e.to_string(), audit_info);
            }
        }

        self.repo.update_in_op(&mut db, &mut csv).await?;
        let (now, tx) = (db.now(), db.into_tx());
        let db_static = es_entity::DbOp::new(tx, now);
        Ok(JobCompletion::CompleteWithOp(db_static))
    }
}

pub struct GenerateCsv<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    ledger_accounts: LedgerAccounts<Perms>,
}

impl<Perms> GenerateCsv<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(ledger_accounts: &LedgerAccounts<Perms>) -> Self {
        Self {
            ledger_accounts: ledger_accounts.clone(),
        }
    }

    pub async fn generate_ledger_account_csv(
        &self,
        ledger_account_id: LedgerAccountId,
    ) -> Result<Vec<u8>, AccountingCsvError> {
        let history_result = self
            .ledger_accounts
            .complete_history(ledger_account_id)
            .await
            .map_err(AccountingCsvError::LedgerAccountError)?;

        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record([
            "Recorded At",
            "Currency",
            "Debit Amount",
            "Credit Amount",
            "Description",
            "Entry Type",
        ])
        .map_err(|e| AccountingCsvError::CsvError(e.to_string()))?;

        for entry in history_result {
            let formatted_amount = entry.amount.to_display_amount();
            let currency = entry.amount.currency_code();

            let (debit_amount, credit_amount) = match entry.direction {
                DebitOrCredit::Debit => (formatted_amount, Decimal::from(0).to_string()),
                DebitOrCredit::Credit => (Decimal::from(0).to_string(), formatted_amount),
            };

            wtr.write_record(&[
                entry.created_at.to_rfc3339(),
                currency,
                debit_amount,
                credit_amount,
                entry.description.unwrap_or_default(),
                entry.entry_type,
            ])
            .map_err(|e| AccountingCsvError::CsvError(e.to_string()))?;
        }
        let csv_data = wtr
            .into_inner()
            .map_err(|e| AccountingCsvError::CsvError(e.to_string()))?;

        Ok(csv_data)
    }
}
