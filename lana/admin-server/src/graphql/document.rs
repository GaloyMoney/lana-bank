use async_graphql::*;

use crate::primitives::*;

pub use lana_app::document::{Document as DomainDocument, DocumentStatus};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Document {
    id: ID,
    document_id: UUID,
    customer_id: Option<UUID>,
    status: DocumentStatus,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDocument>,
}

impl From<DomainDocument> for Document {
    fn from(document: DomainDocument) -> Self {
        Self {
            id: document.id.to_global_id(),
            document_id: UUID::from(document.id),
            customer_id: document.owner_id.map(UUID::from),
            status: document.status,
            entity: Arc::new(document),
        }
    }
}

#[ComplexObject]
impl Document {
    async fn filename(&self) -> &str {
        &self.entity.filename
    }
}

#[derive(InputObject)]
pub struct DocumentCreateInput {
    pub file: Upload,
    pub customer_id: UUID,
}
crate::mutation_payload! { DocumentCreatePayload, document: Document }

#[derive(InputObject)]
pub struct DocumentDownloadLinksGenerateInput {
    pub document_id: UUID,
}

#[derive(SimpleObject)]
pub struct DocumentDownloadLinksGeneratePayload {
    document_id: UUID,
    link: String,
}

impl From<lana_app::document::GeneratedDocumentDownloadLink>
    for DocumentDownloadLinksGeneratePayload
{
    fn from(value: lana_app::document::GeneratedDocumentDownloadLink) -> Self {
        Self {
            document_id: UUID::from(value.document_id),
            link: value.link,
        }
    }
}

#[derive(InputObject)]
pub struct DocumentDeleteInput {
    pub document_id: UUID,
}
#[derive(SimpleObject)]
pub struct DocumentDeletePayload {
    pub deleted_document_id: UUID,
}

#[derive(InputObject)]
pub struct DocumentArchiveInput {
    pub document_id: UUID,
}
crate::mutation_payload! { DocumentArchivePayload, document: Document }
