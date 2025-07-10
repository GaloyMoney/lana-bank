use serde::{Deserialize, Serialize};
use std::borrow::Cow;

es_entity::entity_id! {
    RefTargetId,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Ref(i64);

impl Ref {
    pub fn new(reference: impl Into<String>) -> Self {
        // Parse the string as a number for database storage
        let num = reference.into().parse().unwrap_or(0);
        Ref(num)
    }

    pub fn from_counter(counter: i64) -> Self {
        Ref(counter)
    }
}

impl From<String> for Ref {
    fn from(reference: String) -> Self {
        Ref::new(reference)
    }
}

impl From<i64> for Ref {
    fn from(counter: i64) -> Self {
        Ref(counter)
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
