#![allow(clippy::upper_case_acronyms)]

use async_graphql::*;
use serde::{Deserialize, Serialize};

use domain_config::DomainConfigId;

pub use lana_app::{
    accounting::LedgerAccountId,
    primitives::{
        AccountSpec, ApprovalProcessId, ChartId, CollateralId, CommitteeId, CreditFacilityId,
        CreditFacilityProposalId, CreditFacilityProposalStatus, CustodianId, CustomerDocumentId,
        CustomerId, DepositAccountId, DepositId, DisbursalId, DisbursalStatus, DocumentId, EntryId,
        FiscalYearId, LedgerTransactionId, LiquidationId, ManualTransactionId, PaymentAllocationId,
        PaymentId, PendingCreditFacilityId, PendingCreditFacilityStatus, PermissionSetId, PolicyId,
        ProspectId, ReportId, RoleId, Satoshis, SignedSatoshis, SignedUsdCents, Subject,
        TermsTemplateId, UsdCents, UserId, WalletId, WithdrawalId,
    },
    public_id::PublicId,
    report::ReportRunId,
    terms::{CollateralizationState, PendingCreditFacilityCollateralizationState},
};

pub use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AdminAuthContext {
    pub sub: Subject,
}

impl AdminAuthContext {
    pub fn new(sub: impl Into<UserId>) -> Self {
        Self {
            sub: Subject::User(sub.into()),
        }
    }
}

pub use es_entity::graphql::UUID;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);
scalar!(Timestamp);
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
scalar!(Date);
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

pub trait ToGlobalId {
    fn to_global_id(&self) -> async_graphql::types::ID;
}

macro_rules! impl_to_global_id {
    ($($ty:ty),*) => {
        $(
            impl ToGlobalId for $ty {
                fn to_global_id(&self) -> async_graphql::types::ID {
                    async_graphql::types::ID::from(format!("{}:{}", stringify!($ty).trim_end_matches("Id"), self))
                }
            }
        )*
    };
}

impl_to_global_id! {
    UserId,
    PermissionSetId,
    RoleId,
    CustomerId,
    ChartId,
    TermsTemplateId,
    CreditFacilityProposalId,
    PendingCreditFacilityId,
    CreditFacilityId,
    CollateralId,
    WalletId,
    CustodianId,
    DisbursalId,
    LiquidationId,
    PaymentId,
    audit::AuditEntryId,
    DocumentId,
    CustomerDocumentId,
    PolicyId,
    CommitteeId,
    WithdrawalId,
    DepositId,
    ReportId,
    ReportRunId,
    ManualTransactionId,
    ApprovalProcessId,
    DepositAccountId,
    LedgerTransactionId,
    PaymentAllocationId,
    PublicId,
    EntryId,
    LedgerAccountId,
    FiscalYearId,
    ProspectId,
    DomainConfigId
}
