#![allow(clippy::upper_case_acronyms)]

use async_graphql::*;
use serde::{Deserialize, Serialize};

pub use admin_graphql_shared::primitives::*;
pub use lana_app::terms::{CollateralizationState, PendingCreditFacilityCollateralizationState};

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditEntryId(audit::AuditEntryId);
scalar!(AuditEntryId);
impl From<audit::AuditEntryId> for AuditEntryId {
    fn from(value: audit::AuditEntryId) -> Self {
        Self(value)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditSubjectId(String);
scalar!(AuditSubjectId);
impl From<String> for AuditSubjectId {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<AuditSubjectId> for String {
    fn from(value: AuditSubjectId) -> Self {
        value.0
    }
}
