use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

#[derive(sqlx::Type, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[sqlx(transparent)]
pub struct AuditEntryId(i64);

impl std::fmt::Display for AuditEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<i64> for AuditEntryId {
    fn from(value: i64) -> AuditEntryId {
        AuditEntryId(value)
    }
}

impl From<AuditEntryId> for i64 {
    fn from(value: AuditEntryId) -> i64 {
        value.0
    }
}

pub struct AuditEntry<S, O, A> {
    pub id: AuditEntryId,
    pub subject: S,
    pub object: O,
    pub action: A,
    pub authorized: bool,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AuditInfo {
    pub sub: String,
    pub audit_entry_id: AuditEntryId,
}

impl<T, S> From<(T, S)> for AuditInfo
where
    T: Into<AuditEntryId>,
    S: std::fmt::Display,
{
    fn from((audit_entry_id, sub): (T, S)) -> Self {
        Self {
            sub: sub.to_string(),
            audit_entry_id: audit_entry_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditCursor {
    pub id: AuditEntryId,
}

impl<S, O, A> From<&AuditEntry<S, O, A>> for AuditCursor {
    fn from(entry: &AuditEntry<S, O, A>) -> Self {
        Self { id: entry.id }
    }
}

impl std::fmt::Display for AuditCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl std::str::FromStr for AuditCursor {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<i64>()?;
        Ok(AuditCursor {
            id: AuditEntryId::from(id),
        })
    }
}
