pub mod error;
mod generate;
mod job;
mod primitives;
mod publisher;

use tracing::instrument;

use ::job::JobId;
use audit::AuditSvc;
use authz::PermissionCheck;
use document_storage::{
    Document, DocumentId, DocumentStorage, DocumentType, DocumentsByCreatedAtCursor,
    GeneratedDocumentDownloadLink, ReferenceId,
};
use obix::out::{Outbox, OutboxEventMarker};
use tracing_macros::record_error_severity;

use crate::{Jobs, event::CoreAccountingEvent};

use super::{
    CoreAccountingAction, CoreAccountingObject, ledger_account::LedgerAccounts,
    primitives::LedgerAccountId,
};

use self::job::{
    GenerateAccountingCsvConfig, GenerateAccountingCsvInit, GenerateAccountingCsvJobSpawner,
};
use error::*;
use es_entity::PaginatedQueryArgs;
pub use primitives::*;

pub const LEDGER_ACCOUNT_CSV: DocumentType = DocumentType::new("ledger_account_csv");

pub struct AccountingCsvExports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    authz: Perms,
    generate_accounting_csv_job_spawner: GenerateAccountingCsvJobSpawner<Perms, E>,
    document_storage: DocumentStorage,
}

impl<Perms, E> Clone for AccountingCsvExports<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            generate_accounting_csv_job_spawner: self.generate_accounting_csv_job_spawner.clone(),
            document_storage: self.document_storage.clone(),
        }
    }
}

impl<Perms, E> AccountingCsvExports<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    pub fn new(
        authz: &Perms,
        jobs: &mut Jobs,
        document_storage: DocumentStorage,
        ledger_accounts: &LedgerAccounts<Perms>,
        outbox: &Outbox<E>,
    ) -> Self {
        let generate_accounting_csv_job_spawner = jobs.add_initializer(
            GenerateAccountingCsvInit::new(&document_storage, ledger_accounts, outbox),
        );

        Self {
            authz: authz.clone(),
            generate_accounting_csv_job_spawner,
            document_storage,
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.csv.create", skip(self))]
    pub async fn create_ledger_account_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Document, AccountingCsvExportError> {
        let ledger_account_id = ledger_account_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_CREATE,
            )
            .await?;

        let mut db = self.document_storage.begin_op().await?;
        let document = self
            .document_storage
            .create_in_op(
                &mut db,
                format!("ledger-account-{ledger_account_id}.csv"),
                "text/csv",
                ReferenceId::from(uuid::Uuid::from(ledger_account_id)),
                LEDGER_ACCOUNT_CSV,
            )
            .await?;

        self.generate_accounting_csv_job_spawner
            .spawn_in_op(
                &mut db,
                JobId::from(uuid::Uuid::from(document.id)),
                GenerateAccountingCsvConfig {
                    document_id: document.id,
                    ledger_account_id,
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;
        db.commit().await?;
        Ok(document)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.csv.generate_download_link", skip(self))]
    pub async fn generate_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: DocumentId,
    ) -> Result<GeneratedDocumentDownloadLink, AccountingCsvExportError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let link = self
            .document_storage
            .generate_download_link(document_id)
            .await?;

        Ok(link)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.csv.list_for_ledger_account_id", skip(self))]
    pub async fn list_for_ledger_account_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Vec<Document>, AccountingCsvExportError> {
        let ledger_account_id = ledger_account_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_LIST,
            )
            .await?;

        let documents = self
            .document_storage
            .list_for_reference_id(ReferenceId::from(uuid::Uuid::from(ledger_account_id)))
            .await?;

        Ok(documents)
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.csv.list_for_ledger_account_id_paginated",
        skip(self)
    )]
    pub async fn list_for_ledger_account_id_paginated(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
        query: es_entity::PaginatedQueryArgs<document_storage::DocumentsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Document, document_storage::DocumentsByCreatedAtCursor>,
        AccountingCsvExportError,
    > {
        let ledger_account_id = ledger_account_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_LIST,
            )
            .await?;

        let result = self
            .document_storage
            .list_for_reference_id_paginated(
                ReferenceId::from(uuid::Uuid::from(ledger_account_id)),
                query,
            )
            .await?;

        Ok(result)
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.csv.get_latest_for_ledger_account_id",
        skip(self)
    )]
    pub async fn get_latest_for_ledger_account_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Option<Document>, AccountingCsvExportError> {
        let ledger_account_id = ledger_account_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_LIST,
            )
            .await?;

        let query = PaginatedQueryArgs::<DocumentsByCreatedAtCursor> {
            first: 1,
            after: None,
        };

        let result = self
            .document_storage
            .list_for_reference_id_paginated(
                ReferenceId::from(uuid::Uuid::from(ledger_account_id)),
                query,
            )
            .await?;

        Ok(result.entities.into_iter().next())
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.csv.find_all_documents", skip(self))]
    pub async fn find_all_documents<T: From<Document>>(
        &self,
        ids: &[AccountingCsvDocumentId],
    ) -> Result<std::collections::HashMap<AccountingCsvDocumentId, T>, AccountingCsvExportError>
    {
        let document_ids: Vec<DocumentId> = ids.iter().map(|id| (*id).into()).collect();
        let documents: std::collections::HashMap<DocumentId, T> =
            self.document_storage.find_all(&document_ids).await?;

        let result = documents
            .into_iter()
            .map(|(doc_id, document)| (AccountingCsvDocumentId::from(doc_id), document))
            .collect();

        Ok(result)
    }
}
