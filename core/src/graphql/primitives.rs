#![allow(clippy::upper_case_acronyms)]
use async_graphql::*;
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct UUID(uuid::Uuid);
scalar!(UUID);
impl<T: Into<uuid::Uuid>> From<T> for UUID {
    fn from(id: T) -> Self {
        let uuid = id.into();
        Self(uuid)
    }
}
impl From<&UUID> for FixedTermLoanId {
    fn from(uuid: &UUID) -> Self {
        FixedTermLoanId::from(uuid.0)
    }
}
impl From<UUID> for FixedTermLoanId {
    fn from(uuid: UUID) -> Self {
        FixedTermLoanId::from(uuid.0)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Decimal(rust_decimal::Decimal);
scalar!(Decimal);
impl From<rust_decimal::Decimal> for Decimal {
    fn from(value: rust_decimal::Decimal) -> Self {
        Self(value)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct CurrencyCode(crate::primitives::Currency);
scalar!(CurrencyCode);
impl From<CurrencyCode> for crate::primitives::Currency {
    fn from(code: CurrencyCode) -> Self {
        code.0
    }
}
impl From<crate::primitives::Currency> for CurrencyCode {
    fn from(currency: crate::primitives::Currency) -> Self {
        Self(currency)
    }
}
