use std::{borrow::Cow, fmt::Display};

es_entity::entity_id! {
    NoteId,
    NoteTargetId,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct NoteTargetType(Cow<'static, str>);

impl NoteTargetType {
    pub const fn new(target: &'static str) -> Self {
        NoteTargetType(Cow::Borrowed(target))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for NoteTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
