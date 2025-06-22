use serde::{Deserialize, Serialize};
use std::borrow::Cow;

es_entity::entity_id! {
    DocumentId,
    ReferenceId,
    CustomerId,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct DocumentType(Cow<'static, str>);

impl DocumentType {
    pub const fn new(document_type: &'static str) -> Self {
        DocumentType(Cow::Borrowed(document_type))
    }
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
