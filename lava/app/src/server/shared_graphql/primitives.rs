#![allow(clippy::upper_case_acronyms)]
use async_graphql::*;
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct UUID(uuid::Uuid);
scalar!(UUID);
impl<T: Into<uuid::Uuid>> From<T> for UUID {
    fn from(id: T) -> Self {
        let uuid = id.into();
        Self(uuid)
    }
}
impl From<&UUID> for LoanId {
    fn from(uuid: &UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for LoanId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for CustomerId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<&UUID> for CustomerId {
    fn from(uuid: &UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for DepositId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<&UUID> for DepositId {
    fn from(uuid: &UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for UserId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<&UUID> for UserId {
    fn from(uuid: &UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for WithdrawId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for LedgerAccountId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for LedgerAccountSetId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<&UUID> for ReportId {
    fn from(uuid: &UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for ReportId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<&UUID> for DocumentId {
    fn from(uuid: &UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for DocumentId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<&UUID> for CreditFacilityId {
    fn from(uuid: &UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for CreditFacilityId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}
impl From<UUID> for TermsTemplateId {
    fn from(uuid: UUID) -> Self {
        Self::from(uuid.0)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);
scalar!(Timestamp);
impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }
}
impl Timestamp {
    pub fn into_inner(self) -> chrono::DateTime<chrono::Utc> {
        self.0
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

pub use crate::primitives::Satoshis;
scalar!(Satoshis);

pub use crate::primitives::SignedSatoshis;
scalar!(SignedSatoshis);

pub use crate::primitives::UsdCents;
scalar!(UsdCents);

pub use crate::primitives::SignedUsdCents;
scalar!(SignedUsdCents);
