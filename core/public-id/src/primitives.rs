use serde::{Deserialize, Serialize};
use std::borrow::Cow;

es_entity::entity_id! {
    PublicIdTargetId,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PublicId(String);

#[cfg(feature = "graphql")]
async_graphql::scalar!(PublicId);

impl PublicId {
    pub fn new(id: impl Into<String>) -> Self {
        PublicId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for PublicId {
    fn from(id: String) -> Self {
        PublicId::new(id)
    }
}

impl std::fmt::Display for PublicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PublicIdTargetType(Cow<'static, str>);

impl PublicIdTargetType {
    pub const fn new(target: &'static str) -> Self {
        PublicIdTargetType(Cow::Borrowed(target))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PublicIdTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
