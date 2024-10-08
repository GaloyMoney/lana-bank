use async_graphql::*;

use crate::server::shared_graphql::primitives::*;

#[derive(SimpleObject)]
pub struct Document {
    id: UUID,
    customer_id: UUID,
    filename: String,
}

#[derive(InputObject)]
pub struct DocumentCreateInput {
    pub file: Upload,
    pub customer_id: UUID,
}

#[derive(SimpleObject)]
pub struct DocumentCreatePayload {
    pub document: Document,
}

impl From<crate::document::Document> for Document {
    fn from(document: crate::document::Document) -> Self {
        Self {
            id: UUID::from(document.id),
            customer_id: UUID::from(document.customer_id),
            filename: document.filename,
        }
    }
}

impl From<crate::document::Document> for DocumentCreatePayload {
    fn from(document: crate::document::Document) -> Self {
        Self {
            document: document.into(),
        }
    }
}

// Add this to handle listing documents for a specific customer
#[derive(InputObject)]
pub struct DocumentListForCustomerInput {
    pub customer_id: UUID,
}

#[derive(SimpleObject)]
pub struct DocumentListForCustomerPayload {
    pub documents: Vec<Document>,
}

impl From<Vec<crate::document::Document>> for DocumentListForCustomerPayload {
    fn from(documents: Vec<crate::document::Document>) -> Self {
        Self {
            documents: documents.into_iter().map(Document::from).collect(),
        }
    }
}
