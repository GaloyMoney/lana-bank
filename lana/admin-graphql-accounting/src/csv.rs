use async_graphql::*;

use std::sync::Arc;

use admin_graphql_shared::primitives::*;
pub use lana_app::{
    accounting::csv::AccountingCsvDocumentId,
    document::{Document as DomainDocument, DocumentStatus, GeneratedDocumentDownloadLink},
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct AccountingCsvDocument {
    id: ID,
    document_id: UUID,
    ledger_account_id: UUID,
    status: DocumentStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainDocument>,
}

impl AccountingCsvDocument {
    pub fn accounting_csv_document_id(&self) -> AccountingCsvDocumentId {
        AccountingCsvDocumentId::from(self.entity.id)
    }
}

impl From<DomainDocument> for AccountingCsvDocument {
    fn from(document: DomainDocument) -> Self {
        Self {
            id: document.id.to_global_id(),
            document_id: UUID::from(document.id),
            ledger_account_id: UUID::from(document.reference_id),
            status: document.status,
            created_at: document.created_at().into(),
            entity: Arc::new(document),
        }
    }
}

#[ComplexObject]
impl AccountingCsvDocument {
    async fn filename(&self) -> &str {
        &self.entity.filename
    }
}

#[derive(SimpleObject)]
pub struct AccountingCsvDownloadLink {
    pub url: String,
    pub csv_id: UUID,
}

impl From<GeneratedDocumentDownloadLink> for AccountingCsvDownloadLink {
    fn from(result: GeneratedDocumentDownloadLink) -> Self {
        Self {
            url: result.link,
            csv_id: UUID::from(result.document_id),
        }
    }
}

#[derive(SimpleObject)]
pub struct LedgerAccountCsvExportUploadedPayload {
    pub document_id: UUID,
}

#[derive(InputObject)]
pub struct LedgerAccountCsvCreateInput {
    pub ledger_account_id: UUID,
}
mutation_payload! { LedgerAccountCsvCreatePayload, accounting_csv_document: AccountingCsvDocument }

#[derive(InputObject)]
pub struct AccountingCsvDownloadLinkGenerateInput {
    pub document_id: UUID,
}
mutation_payload! { AccountingCsvDownloadLinkGeneratePayload, link: AccountingCsvDownloadLink }
