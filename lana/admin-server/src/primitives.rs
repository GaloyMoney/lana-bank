#![allow(clippy::upper_case_acronyms)]

use async_graphql::*;
use serde::{Deserialize, Serialize};

pub use lana_app::price::PriceProviderId;

pub use lana_app::{
    accounting::LedgerAccountId,
    primitives::{
        ApprovalProcessId, ChartId, CollateralDirection, CollateralId, CommitteeId,
        CreditFacilityId, CreditFacilityProposalId, CreditFacilityProposalStatus, CustodianId,
        CustomerDocumentId, CustomerId, DepositAccountId, DepositId, DisbursalId, DisbursalStatus,
        FiscalYearId, LedgerTransactionId, LiquidationId, PendingCreditFacilityId,
        PendingCreditFacilityStatus, PermissionSetId, PolicyId, ProspectId, RoleId, Satoshis,
        SignedSatoshis, SignedUsdCents, Subject, TermsTemplateId, UsdCents, UserId, WalletId,
        WithdrawalId,
    },
    terms::{CollateralizationState, PendingCreditFacilityCollateralizationState},
};

pub use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AdminAuthContext {
    pub sub: Subject,
    pub token_auth_time: Option<chrono::DateTime<chrono::Utc>>,
}

const STEP_UP_AUTH_MAX_AGE_SECS: i64 = 60;

impl AdminAuthContext {
    pub fn new(sub: impl Into<UserId>, auth_time: Option<i64>, original_iat: Option<i64>) -> Self {
        let token_auth_time = auth_time
            .or(original_iat)
            .and_then(|t| chrono::DateTime::from_timestamp(t, 0));
        Self {
            sub: Subject::User(sub.into()),
            token_auth_time,
        }
    }

    pub fn enforce_step_up_auth(&self) -> async_graphql::Result<()> {
        let auth_time = self.token_auth_time.ok_or_else(|| {
            async_graphql::Error::new("Step-up authentication required: auth_time claim missing")
        })?;

        let age = chrono::Utc::now()
            .signed_duration_since(auth_time)
            .num_seconds();

        if age > STEP_UP_AUTH_MAX_AGE_SECS {
            return Err(async_graphql::Error::new(format!(
                "Step-up authentication required: token too old ({}s > {}s)",
                age, STEP_UP_AUTH_MAX_AGE_SECS
            )));
        }

        Ok(())
    }
}

pub use es_entity::graphql::UUID;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);
scalar!(
    Timestamp,
    "Timestamp",
    "An ISO 8601 UTC timestamp (e.g., 2024-01-15T09:30:00Z). Always in UTC."
);
impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }
}
impl Timestamp {
    #[allow(dead_code)]
    pub fn into_inner(self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Date(chrono::NaiveDate);
scalar!(
    Date,
    "Date",
    "An ISO 8601 calendar date without time or timezone (e.g., 2024-01-15). Represents a business date; timezone-naive by design."
);
impl From<chrono::NaiveDate> for Date {
    fn from(value: chrono::NaiveDate) -> Self {
        Self(value)
    }
}
impl From<Date> for chrono::NaiveDate {
    fn from(value: Date) -> Self {
        value.0
    }
}
impl Date {
    #[allow(dead_code)]
    pub fn into_inner(self) -> chrono::NaiveDate {
        self.0
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditEntryId(audit::AuditEntryId);
scalar!(AuditEntryId);
impl From<audit::AuditEntryId> for AuditEntryId {
    fn from(value: audit::AuditEntryId) -> Self {
        Self(value)
    }
}
impl std::fmt::Display for AuditEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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

#[derive(Clone, Serialize)]
#[serde(transparent)]
pub struct ReportDefinitionId(lana_app::report::ReportDefinitionId);
scalar!(ReportDefinitionId);
impl From<lana_app::report::ReportDefinitionId> for ReportDefinitionId {
    fn from(value: lana_app::report::ReportDefinitionId) -> Self {
        Self(value)
    }
}
impl From<ReportDefinitionId> for lana_app::report::ReportDefinitionId {
    fn from(value: ReportDefinitionId) -> Self {
        value.0
    }
}
impl std::fmt::Display for ReportDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<'de> Deserialize<'de> for ReportDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        lana_app::report::ReportDefinitionId::try_new(s)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
