use async_graphql::{connection::*, *};

use crate::primitives::*;

use super::event_timeline::{self, EventTimelineCursor, EventTimelineEntry};
pub use lana_app::document::{Document as DomainDocument, DocumentStatus};

#[derive(SimpleObject, Clone)]
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("customerDocumentId".to_string())
)]
pub struct CustomerDocument {
    customer_document_id: CustomerDocumentId,
    customer_id: CustomerId,
    status: DocumentStatus,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDocument>,
}

impl From<DomainDocument> for CustomerDocument {
    fn from(document: DomainDocument) -> Self {
        Self {
            customer_document_id: document.id.into(),
            customer_id: document.reference_id.into(),
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
        use es_entity::EsEntity as _;
        event_timeline::events_to_connection(self.entity.events(), first, after)
    }
}

#[derive(InputObject)]
pub struct CustomerDocumentCreateInput {
    pub file: Upload,
    pub customer_id: CustomerId,
}
crate::mutation_payload! { CustomerDocumentCreatePayload, customer_document: CustomerDocument }

#[derive(InputObject)]
pub struct CustomerDocumentDownloadLinksGenerateInput {
    pub customer_document_id: CustomerDocumentId,
}

#[derive(SimpleObject)]
pub struct CustomerDocumentDownloadLinksGeneratePayload {
    link: String,
}

impl From<lana_app::document::GeneratedDocumentDownloadLink>
    for CustomerDocumentDownloadLinksGeneratePayload
{
    fn from(value: lana_app::document::GeneratedDocumentDownloadLink) -> Self {
        Self { link: value.link }
    }
}

#[derive(InputObject)]
pub struct CustomerDocumentDeleteInput {
    pub customer_document_id: CustomerDocumentId,
}
#[derive(SimpleObject)]
pub struct CustomerDocumentDeletePayload {
    pub deleted_document_id: CustomerDocumentId,
}

#[derive(InputObject)]
pub struct CustomerDocumentArchiveInput {
    pub customer_document_id: CustomerDocumentId,
}
crate::mutation_payload! { CustomerDocumentArchivePayload, customer_document: CustomerDocument }
