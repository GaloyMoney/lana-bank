use serde::{Deserialize, Serialize};
use std::borrow::Cow;

es_entity::entity_id! {
    PublicIdTargetId,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Id(String);

impl Id {
    pub fn new(id: impl Into<String>) -> Self {
        Id(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Id::new(id)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct IdTargetType(Cow<'static, str>);

impl IdTargetType {
    pub const fn new(target: &'static str) -> Self {
        IdTargetType(Cow::Borrowed(target))
    }
}

impl std::fmt::Display for IdTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
