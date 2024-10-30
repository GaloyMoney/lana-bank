#![allow(clippy::upper_case_acronyms)]

use async_graphql::*;
use serde::{Deserialize, Serialize};

pub use lava_app::primitives::{
    AccountStatus, ApprovalProcessId, CommitteeId, CustomerId, DocumentId, KycLevel, LavaRole,
    PolicyId, Subject, UserId,
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
// impl Timestamp {
//     pub fn into_inner(self) -> chrono::DateTime<chrono::Utc> {
//         self.0
//     }
// }

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalProcessType {
    WithdrawApproval,
    CreditFacilityApproval,
}

impl From<&governance::ApprovalProcessType> for ApprovalProcessType {
    fn from(process_type: &governance::ApprovalProcessType) -> Self {
        if process_type == &lava_app::governance::APPROVE_WITHDRAW_PROCESS {
            Self::WithdrawApproval
        } else if process_type == &lava_app::governance::APPROVE_CREDIT_FACILITY_PROCESS {
            Self::CreditFacilityApproval
        } else {
            panic!("Unknown ApprovalProcessType")
        }
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
    CustomerId,
    audit::AuditEntryId,
    DocumentId,
    PolicyId,
    CommitteeId,
    ApprovalProcessId
}
