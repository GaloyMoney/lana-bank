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
    ledger_account_csv_document_id: UUID,
    ledger_account_id: UUID,
    status: DocumentStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainDocument>,
}

impl From<DomainDocument> for LedgerAccountCsvDocument {
    fn from(document: DomainDocument) -> Self {
        Self {
            ledger_account_csv_document_id: UUID::from(document.id),
            ledger_account_id: UUID::from(document.reference_id),
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
    pub csv_id: UUID,
}

impl From<GeneratedDocumentDownloadLink> for LedgerAccountCsvDownloadLink {
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
crate::mutation_payload! { LedgerAccountCsvCreatePayload, ledger_account_csv_document: LedgerAccountCsvDocument }

#[derive(InputObject)]
pub struct LedgerAccountCsvDownloadLinkGenerateInput {
    pub document_id: UUID,
}
crate::mutation_payload! { LedgerAccountCsvDownloadLinkGeneratePayload, link: LedgerAccountCsvDownloadLink }
