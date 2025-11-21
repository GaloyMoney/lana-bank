use serde::{Deserialize, Serialize};
use sqlx::{Postgres, postgres::{PgArgumentBuffer, PgValueRef}};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

es_entity::entity_id!(FundingLinkId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LinkStatus {
    Active,
    Inactive,
    Broken,
}

impl sqlx::Type<Postgres> for LinkStatus {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, Postgres> for LinkStatus {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, Postgres> for LinkStatus {
    fn decode(value: PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
        match s.as_str() {
            "active" => Ok(LinkStatus::Active),
            "inactive" => Ok(LinkStatus::Inactive),
            "broken" => Ok(LinkStatus::Broken),
            _ => Err(format!("Invalid LinkStatus: {}", s).into()),
        }
    }
}

impl std::fmt::Display for LinkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkStatus::Active => write!(f, "active"),
            LinkStatus::Inactive => write!(f, "inactive"),
            LinkStatus::Broken => write!(f, "broken"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum BrokenReason {
    AccountClosed,
    AccountFrozen,
    Manual,
}

impl std::fmt::Display for BrokenReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrokenReason::AccountClosed => write!(f, "account_closed"),
            BrokenReason::AccountFrozen => write!(f, "account_frozen"),
            BrokenReason::Manual => write!(f, "manual"),
        }
    }
}

impl sqlx::Type<Postgres> for BrokenReason {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, Postgres> for BrokenReason {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <String as sqlx::Encode<Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, Postgres> for BrokenReason {
    fn decode(value: PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
        match s.as_str() {
            "account_closed" => Ok(BrokenReason::AccountClosed),
            "account_frozen" => Ok(BrokenReason::AccountFrozen),
            "manual" => Ok(BrokenReason::Manual),
            _ => Err(format!("Invalid BrokenReason: {}", s).into()),
        }
    }
}

