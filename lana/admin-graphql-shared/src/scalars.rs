use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl From<SortDirection> for es_entity::ListDirection {
    fn from(direction: SortDirection) -> Self {
        match direction {
            SortDirection::Asc => Self::Ascending,
            SortDirection::Desc => Self::Descending,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Decimal(rust_decimal::Decimal);
async_graphql::scalar!(Decimal);
impl From<rust_decimal::Decimal> for Decimal {
    fn from(value: rust_decimal::Decimal) -> Self {
        Self(value)
    }
}
impl From<Decimal> for rust_decimal::Decimal {
    fn from(value: Decimal) -> Self {
        value.0
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Json(Value);
async_graphql::scalar!(Json);
impl From<Value> for Json {
    fn from(value: Value) -> Self {
        Self(value)
    }
}
impl From<Json> for Value {
    fn from(value: Json) -> Self {
        value.0
    }
}
impl Json {
    pub fn into_inner(self) -> Value {
        self.0
    }
}
