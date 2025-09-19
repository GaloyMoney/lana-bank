use async_graphql::*;
use serde::{Deserialize, Serialize};

pub use std::sync::Arc;

pub use lana_app::{
    primitives::{
        CreditFacilityId, CreditFacilityStatus, CustomerId, DepositAccountId, DepositId,
        DisbursalId, DisbursalStatus, PaymentAllocationId, Satoshis, Subject, UsdCents,
        WithdrawalId,
    },
    terms::CollateralizationState,
};

pub use es_entity::graphql::UUID;

#[derive(Debug, Clone)]
pub struct CustomerAuthContext {
    pub sub: Subject,
}

impl CustomerAuthContext {
    pub(crate) fn new(sub: impl Into<CustomerId>) -> Self {
        Self {
            sub: Subject::Customer(sub.into()),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);
scalar!(Timestamp);
impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
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
    pub(crate) fn into_inner(self) -> chrono::NaiveDate {
        self.0
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
    CustomerId,
    DepositAccountId,
    DepositId,
    WithdrawalId,
    CreditFacilityId,
    DisbursalId,
    PaymentAllocationId
}
