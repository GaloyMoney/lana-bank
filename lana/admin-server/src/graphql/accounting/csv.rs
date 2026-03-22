use async_graphql::*;

use crate::primitives::*;
pub use lana_app::document::{
    Document as DomainDocument, DocumentStatus, GeneratedDocumentDownloadLink,
};
use std::sync::Arc;

#[derive(SimpleObject, Clone)]
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("ledgerAccountCsvDocumentId".to_string())
)]
pub struct LedgerAccountCsvDocument {
    ledger_account_csv_document_id: AccountingCsvDocumentId,
    ledger_account_id: LedgerAccountId,
    status: DocumentStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainDocument>,
}

impl From<DomainDocument> for LedgerAccountCsvDocument {
    fn from(document: DomainDocument) -> Self {
        Self {
            ledger_account_csv_document_id: AccountingCsvDocumentId::from(uuid::Uuid::from(
                document.id,
            )),
            ledger_account_id: LedgerAccountId::from(uuid::Uuid::from(document.reference_id)),
            status: document.status,
            created_at: document.created_at().into(),
            entity: Arc::new(document),
        }
    }
}

#[ComplexObject]
impl LedgerAccountCsvDocument {
    async fn filename(&self) -> &str {
        &self.entity.filename
    }
}

#[derive(SimpleObject)]
pub struct LedgerAccountCsvDownloadLink {
    pub url: String,
    pub csv_id: AccountingCsvDocumentId,
}

impl From<GeneratedDocumentDownloadLink> for LedgerAccountCsvDownloadLink {
    fn from(result: GeneratedDocumentDownloadLink) -> Self {
        Self {
            url: result.link,
            csv_id: result.document_id.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct LedgerAccountCsvExportUploadedPayload {
    pub document_id: AccountingCsvDocumentId,
}

#[derive(InputObject)]
pub struct LedgerAccountCsvCreateInput {
    pub ledger_account_id: LedgerAccountId,
}
crate::mutation_payload! { LedgerAccountCsvCreatePayload, ledger_account_csv_document: LedgerAccountCsvDocument }

#[derive(InputObject)]
pub struct LedgerAccountCsvDownloadLinkGenerateInput {
    pub ledger_account_csv_document_id: AccountingCsvDocumentId,
}
crate::mutation_payload! { LedgerAccountCsvDownloadLinkGeneratePayload, link: LedgerAccountCsvDownloadLink }
