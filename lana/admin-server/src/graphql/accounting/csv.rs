use async_graphql::*;

use crate::primitives::*;
pub use document_storage::{Document as DomainDocument, DocumentStatus};
pub use lana_app::accounting::csv::AccountingCsvDocumentId;
use std::sync::Arc;

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

impl From<document_storage::GeneratedDocumentDownloadLink> for AccountingCsvDownloadLink {
    fn from(result: document_storage::GeneratedDocumentDownloadLink) -> Self {
        Self {
            url: result.link,
            csv_id: UUID::from(result.document_id),
        }
    }
}

#[derive(InputObject)]
pub struct LedgerAccountCsvCreateInput {
    pub ledger_account_id: UUID,
}
crate::mutation_payload! { LedgerAccountCsvCreatePayload, accounting_csv_document: AccountingCsvDocument }

#[derive(InputObject)]
pub struct AccountingCsvDownloadLinkGenerateInput {
    pub document_id: UUID,
}
crate::mutation_payload! { AccountingCsvDownloadLinkGeneratePayload, link: AccountingCsvDownloadLink }
