use async_graphql::{connection::*, *};
use es_entity::EsEntity as _;

use crate::primitives::*;

use super::event_timeline::{self, EventTimelineCursor, EventTimelineEntry};
pub use lana_app::document::{Document as DomainDocument, DocumentStatus};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CustomerDocument {
    id: ID,
    customer_document_id: UUID,
    customer_id: UUID,
    status: DocumentStatus,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDocument>,
}

impl From<DomainDocument> for CustomerDocument {
    fn from(document: DomainDocument) -> Self {
        Self {
            id: document.id.to_global_id(),
            customer_document_id: UUID::from(document.id),
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

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        event_timeline::events_to_connection(self.entity.events(), first, after)
    }
}

#[derive(InputObject)]
pub struct CustomerDocumentCreateInput {
    pub file: Upload,
    pub customer_id: UUID,
}
crate::mutation_payload! { CustomerDocumentCreatePayload, document: CustomerDocument }

#[derive(InputObject)]
pub struct CustomerDocumentDownloadLinksGenerateInput {
    pub document_id: UUID,
}

#[derive(SimpleObject)]
pub struct CustomerDocumentDownloadLinksGeneratePayload {
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
pub struct CustomerDocumentDeleteInput {
    pub document_id: UUID,
}
#[derive(SimpleObject)]
pub struct CustomerDocumentDeletePayload {
    pub deleted_document_id: UUID,
}

#[derive(InputObject)]
pub struct CustomerDocumentArchiveInput {
    pub document_id: UUID,
}
crate::mutation_payload! { CustomerDocumentArchivePayload, document: CustomerDocument }
