use async_trait::async_trait;

use authz::PermissionCheck;

use audit::AuditSvc;
use document_storage::{CoreDocumentStorageEvent, DocumentId, DocumentStorage};
use job::*;
use obix::out::OutboxEventMarker;
use serde::{Deserialize, Serialize};

use crate::{ledger_account::LedgerAccounts, primitives::LedgerAccountId};

use super::{CoreAccountingAction, CoreAccountingObject, generate::GenerateCsvExport};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateAccountingCsvConfig<Perms> {
    pub document_id: DocumentId,
    pub ledger_account_id: LedgerAccountId,
    pub _phantom: std::marker::PhantomData<Perms>,
}

pub struct GenerateAccountingCsvInit<Perms, E>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    document_storage: DocumentStorage<E>,
    ledger_accounts: LedgerAccounts<Perms>,
}

impl<Perms, E> GenerateAccountingCsvInit<Perms, E>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    pub fn new(
        document_storage: &DocumentStorage<E>,
        ledger_accounts: &LedgerAccounts<Perms>,
    ) -> Self {
        Self {
            document_storage: document_storage.clone(),
            ledger_accounts: ledger_accounts.clone(),
        }
    }
}

pub const GENERATE_ACCOUNTING_CSV_JOB: JobType = JobType::new("task.generate-accounting-csv");

impl<Perms, E> JobInitializer for GenerateAccountingCsvInit<Perms, E>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    type Config = GenerateAccountingCsvConfig<Perms>;
    fn job_type(&self) -> JobType {
        GENERATE_ACCOUNTING_CSV_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(GenerateAccountingCsvExportJobRunner {
            config: job.config()?,
            document_storage: self.document_storage.clone(),
            generator: GenerateCsvExport::new(&self.ledger_accounts),
        }))
    }
}

pub struct GenerateAccountingCsvExportJobRunner<Perms, E>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    config: GenerateAccountingCsvConfig<Perms>,
    document_storage: DocumentStorage<E>,
    generator: GenerateCsvExport<Perms>,
}

#[async_trait]
impl<Perms, E> JobRunner for GenerateAccountingCsvExportJobRunner<Perms, E>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let csv_result = self
            .generator
            .generate_ledger_account_csv(self.config.ledger_account_id)
            .await?;

        let document_id = self.config.document_id;
        let mut document = self.document_storage.find_by_id(document_id).await?;

        self.document_storage
            .upload(csv_result, &mut document)
            .await?;

        Ok(JobCompletion::Complete)
    }
}

pub type GenerateAccountingCsvJobSpawner<Perms> = JobSpawner<GenerateAccountingCsvConfig<Perms>>;
