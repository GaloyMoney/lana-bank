mod entity;
pub mod error;
mod generate;
mod job;
mod primitives;
mod repo;

use tracing::instrument;

use ::job::JobId;
use audit::AuditSvc;
use authz::PermissionCheck;
use document_storage::{
    Document, DocumentId, DocumentStorage, DocumentType, GeneratedDocumentDownloadLink, ReferenceId,
};

use crate::Jobs;

use super::{
    CoreAccountingAction, CoreAccountingObject, ledger_account::LedgerAccounts,
    primitives::LedgerAccountId,
};

#[cfg(feature = "json-schema")]
pub use entity::AccountingCsvEvent;
pub use entity::{AccountingCsv, NewAccountingCsv};
use error::*;
use job::*;
pub use primitives::*;
// Note: Using EntityByCreatedAtCursor from es_entity instead of custom cursor since we now use Document storage
use repo::*;

pub const LEDGER_ACCOUNT_CSV: DocumentType = DocumentType::new("ledger_account_csv");

#[derive(Clone)]
pub struct AccountingCsvs<Perms>
where
    Perms: PermissionCheck,
{
    repo: AccountingCsvRepo,
    authz: Perms,
    jobs: Jobs,
    document_storage: DocumentStorage,
}

impl<Perms> AccountingCsvs<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        jobs: &Jobs,
        document_storage: DocumentStorage,
        ledger_accounts: &LedgerAccounts<Perms>,
    ) -> Self {
        let repo = AccountingCsvRepo::new(pool);

        jobs.add_initializer(GenerateAccountingCsvInit::new(
            &document_storage,
            ledger_accounts,
            authz.audit(),
        ));

        Self {
            repo,
            authz: authz.clone(),
            jobs: jobs.clone(),
            document_storage,
        }
    }

    #[instrument(name = "core_accounting.csv.create", skip(self), err)]
    pub async fn create_ledger_account_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Document, AccountingCsvError> {
        let ledger_account_id = ledger_account_id.into();

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_CREATE,
            )
            .await?;

        // Create document using document storage instead of AccountingCsv entity
        let document = self
            .document_storage
            .create(
                audit_info.clone(),
                format!("ledger-account-{}.csv", ledger_account_id),
                "text/csv",
                ReferenceId::from(uuid::Uuid::from(ledger_account_id)),
                LEDGER_ACCOUNT_CSV,
            )
            .await?;

        // Create and spawn job to generate and upload the CSV content
        let mut db = self.repo.begin_op().await?;
        self.jobs
            .create_and_spawn_in_op::<GenerateAccountingCsvConfig<Perms>>(
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

    #[instrument(name = "core_accounting.csv.generate_download_link", skip(self), err)]
    pub async fn generate_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: DocumentId,
    ) -> Result<GeneratedDocumentDownloadLink, AccountingCsvError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let link = self
            .document_storage
            .generate_download_link(audit_info, document_id)
            .await?;

        Ok(link)
    }

    #[instrument(
        name = "core_accounting.csv.list_for_ledger_account_id",
        skip(self),
        err
    )]
    pub async fn list_for_ledger_account_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Vec<Document>, AccountingCsvError> {
        let ledger_account_id = ledger_account_id.into();

        let _audit_info = self
            .authz
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

    #[instrument(
        name = "core_accounting.csv.list_for_ledger_account_id_paginated",
        skip(self),
        err
    )]
    pub async fn list_for_ledger_account_id_paginated(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
        query: es_entity::PaginatedQueryArgs<document_storage::DocumentsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Document, document_storage::DocumentsByCreatedAtCursor>,
        AccountingCsvError,
    > {
        let ledger_account_id = ledger_account_id.into();

        let _audit_info = self
            .authz
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

    #[instrument(name = "core_accounting.csv.find_all_documents", skip(self), err)]
    pub async fn find_all_documents<T: From<Document>>(
        &self,
        ids: &[AccountingCsvDocumentId],
    ) -> Result<std::collections::HashMap<AccountingCsvDocumentId, T>, AccountingCsvError> {
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
