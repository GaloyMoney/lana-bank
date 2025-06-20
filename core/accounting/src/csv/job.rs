use async_trait::async_trait;

use authz::PermissionCheck;

use audit::AuditSvc;
use document_storage::{DocumentId, DocumentStorage};
use job::*;
use serde::{Deserialize, Serialize};

use crate::{ledger_account::LedgerAccounts, primitives::LedgerAccountId};

use super::{CoreAccountingAction, CoreAccountingObject, generate::GenerateCsv};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateAccountingCsvConfig<Perms> {
    pub document_id: DocumentId,
    pub ledger_account_id: LedgerAccountId,
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
    document_storage: DocumentStorage,
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
        document_storage: &DocumentStorage,
        ledger_accounts: &LedgerAccounts<Perms>,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            document_storage: document_storage.clone(),
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
            document_storage: self.document_storage.clone(),
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
    document_storage: DocumentStorage,
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
        let audit_info = self
            .audit
            .record_system_entry(
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_GENERATE,
            )
            .await?;

        // Generate CSV data
        let csv_result = self
            .generator
            .generate_ledger_account_csv(self.config.ledger_account_id)
            .await;

        match csv_result {
            Ok(csv_data) => {
                // Find the document and upload content
                if let Some(mut document) = self
                    .document_storage
                    .find_by_id(self.config.document_id)
                    .await?
                {
                    match self
                        .document_storage
                        .upload(csv_data, &mut document, audit_info)
                        .await
                    {
                        Ok(_) => {
                            // Upload successful
                        }
                        Err(e) => {
                            // Upload failed - document storage will handle the error state
                            return Err(e.into());
                        }
                    }
                } else {
                    return Err("Document not found".into());
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        Ok(JobCompletion::Complete)
    }
}
