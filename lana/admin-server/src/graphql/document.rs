use async_graphql::*;

use crate::primitives::*;
use lana_app::customer::CustomerDocumentId;

pub(super) use lana_app::document::{Document as DomainDocument, DocumentStatus};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CustomerDocument {
    id: ID,
    document_id: UUID,
    customer_id: UUID,
    status: DocumentStatus,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDocument>,
}

impl CustomerDocument {
    pub fn customer_document_id(&self) -> CustomerDocumentId {
        CustomerDocumentId::from(self.entity.id)
    }
}

impl From<DomainDocument> for CustomerDocument {
    fn from(document: DomainDocument) -> Self {
        Self {
            id: document.id.to_global_id(),
            document_id: UUID::from(document.id),
            customer_id: UUID::from(document.reference_id),
            status: document.status,
            entity: Arc::new(document),
        }
    }
}

#[ComplexObject]
impl CustomerDocument {
    async fn filename(&self) -> &str {
        &self.entity.filename
    }
}

#[derive(InputObject)]
pub struct CustomerDocumentCreateInput {
    pub file: Upload,
    pub customer_id: UUID,
}
crate::mutation_payload! { CustomerDocumentCreatePayload, document: CustomerDocument }

#[derive(InputObject)]
pub(super) struct CustomerDocumentDownloadLinksGenerateInput {
    pub document_id: UUID,
}

#[derive(SimpleObject)]
pub(super) struct CustomerDocumentDownloadLinksGeneratePayload {
    document_id: UUID,
    link: String,
}

impl From<lana_app::document::GeneratedDocumentDownloadLink>
    for CustomerDocumentDownloadLinksGeneratePayload
{
    fn from(value: lana_app::document::GeneratedDocumentDownloadLink) -> Self {
        Self {
            document_id: UUID::from(value.document_id),
            link: value.link,
        }
    }
}

#[derive(InputObject)]
pub(super) struct CustomerDocumentDeleteInput {
    pub document_id: UUID,
}
#[derive(SimpleObject)]
pub(super) struct CustomerDocumentDeletePayload {
    pub deleted_document_id: UUID,
}

#[derive(InputObject)]
pub(super) struct CustomerDocumentArchiveInput {
    pub document_id: UUID,
}
crate::mutation_payload! { CustomerDocumentArchivePayload, document: CustomerDocument }
