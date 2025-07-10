use serde::{Deserialize, Serialize};
use std::borrow::Cow;

es_entity::entity_id! {
    RefTargetId,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Ref(String);

impl Ref {
    pub fn new(reference: impl Into<String>) -> Self {
        Ref(reference.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Ref {
    fn from(reference: String) -> Self {
        Ref::new(reference)
    }
}

impl std::fmt::Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RefTargetType(Cow<'static, str>);

impl RefTargetType {
    pub const fn new(target: &'static str) -> Self {
        RefTargetType(Cow::Borrowed(target))
    }
}

impl std::fmt::Display for RefTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
