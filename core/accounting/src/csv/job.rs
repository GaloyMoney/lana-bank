use async_trait::async_trait;

use authz::PermissionCheck;

use audit::AuditSvc;
use document_storage::{DocumentId, DocumentStorage};
use job::*;
use obix::out::{Outbox, OutboxEventMarker};
use serde::{Deserialize, Serialize};

use crate::event::CoreAccountingEvent;
use crate::primitives::AccountingCsvId;
use crate::{ledger_account::LedgerAccounts, primitives::LedgerAccountId};

use super::publisher::AccountingCsvPublisher;
use super::{CoreAccountingAction, CoreAccountingObject, generate::GenerateCsvExport};

#[derive(Serialize, Deserialize)]
pub struct GenerateAccountingCsvConfig<Perms, E> {
    pub document_id: DocumentId,
    pub ledger_account_id: LedgerAccountId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for GenerateAccountingCsvConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            document_id: self.document_id,
            ledger_account_id: self.ledger_account_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct GenerateAccountingCsvInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    document_storage: DocumentStorage,
    ledger_accounts: LedgerAccounts<Perms>,
    publisher: AccountingCsvPublisher<E>,
}

impl<Perms, E> GenerateAccountingCsvInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    pub fn new(
        document_storage: &DocumentStorage,
        ledger_accounts: &LedgerAccounts<Perms>,
        outbox: &Outbox<E>,
    ) -> Self {
        Self {
            document_storage: document_storage.clone(),
            ledger_accounts: ledger_accounts.clone(),
            publisher: AccountingCsvPublisher::new(outbox),
        }
    }
}

pub const GENERATE_ACCOUNTING_CSV_JOB: JobType = JobType::new("task.generate-accounting-csv");

impl<Perms, E> JobInitializer for GenerateAccountingCsvInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    type Config = GenerateAccountingCsvConfig<Perms, E>;
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
            publisher: self.publisher.clone(),
        }))
    }
}

pub struct GenerateAccountingCsvExportJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    config: GenerateAccountingCsvConfig<Perms, E>,
    document_storage: DocumentStorage,
    generator: GenerateCsvExport<Perms>,
    publisher: AccountingCsvPublisher<E>,
}

#[async_trait]
impl<Perms, E> JobRunner for GenerateAccountingCsvExportJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let csv_result = self
            .generator
            .generate_ledger_account_csv(self.config.ledger_account_id)
            .await?;

        let document_id = self.config.document_id;

        let mut op = current_job.begin_op().await?;
        let mut document = self
            .document_storage
            .find_by_id_in_op(&mut op, document_id)
            .await?;
        self.document_storage
            .upload_in_op(&mut op, csv_result, &mut document)
            .await?;

        let csv_id = AccountingCsvId::from(uuid::Uuid::from(document.id));
        self.publisher
            .publish_csv_export_uploaded_in_op(&mut op, csv_id, self.config.ledger_account_id)
            .await?;
        op.commit().await?;

        Ok(JobCompletion::Complete)
    }
}

pub type GenerateAccountingCsvJobSpawner<Perms, E> =
    JobSpawner<GenerateAccountingCsvConfig<Perms, E>>;
