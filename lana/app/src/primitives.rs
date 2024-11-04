use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::fmt;
use thiserror::Error;

pub use core_user::UserId;
pub use governance::{ApprovalProcessId, CommitteeId, CommitteeMemberId, PolicyId};
pub use job::JobId;
pub use lana_ids::*;
pub use rbac_types::{LanaRole, Role, Subject};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DisbursalIdx(i32);
async_graphql::scalar!(DisbursalIdx);

impl fmt::Display for DisbursalIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DisbursalIdx {
    pub const FIRST: Self = Self(1);
    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct InterestAccrualIdx(i32);
impl fmt::Display for InterestAccrualIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl InterestAccrualIdx {
    pub const FIRST: Self = Self(1);
    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

// Consider importing from cala
#[derive(Debug)]
pub enum LedgerAccountSetMemberType {
    Account,
    AccountSet,
}

#[derive(async_graphql::Enum, Debug, Deserialize, Clone, Copy, Serialize, Eq, PartialEq)]
pub enum KycLevel {
    NotKyced,
    Basic,
    Advanced,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoanStatus {
    New,
    Active,
    Closed,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreditFacilityStatus {
    PendingCollateralization,
    PendingApproval,
    Active,
    Expired,
    Closed,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisbursalStatus {
    New,
    Approved,
    Denied,
    Confirmed,
}

impl std::fmt::Display for KycLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KycLevel::NotKyced => write!(f, "not-kyc"),
            KycLevel::Basic => write!(f, "basic-kyc-level"),
            KycLevel::Advanced => write!(f, "advanced-kyc-level"),
        }
    }
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    Active,
    Inactive,
}

pub use cala_types::primitives::{
    AccountId as LedgerAccountId, AccountSetId as LedgerAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};

pub const SATS_PER_BTC: Decimal = dec!(100_000_000);
pub const CENTS_PER_USD: Decimal = dec!(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignedSatoshis(i64);
async_graphql::scalar!(SignedSatoshis);

impl From<Satoshis> for SignedSatoshis {
    fn from(sats: Satoshis) -> Self {
        Self(i64::try_from(sats.0).expect("Satoshis must be integer sized for i64"))
    }
}

impl fmt::Display for SignedSatoshis {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SignedSatoshis {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Sub<SignedSatoshis> for SignedSatoshis {
    type Output = SignedSatoshis;

    fn sub(self, other: SignedSatoshis) -> SignedSatoshis {
        SignedSatoshis(self.0 - other.0)
    }
}

impl std::ops::Add<SignedSatoshis> for SignedSatoshis {
    type Output = SignedSatoshis;

    fn add(self, other: SignedSatoshis) -> SignedSatoshis {
        SignedSatoshis(self.0 + other.0)
    }
}

impl SignedSatoshis {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_btc(self) -> Decimal {
        Decimal::from(self.0) / SATS_PER_BTC
    }

    pub fn abs(self) -> SignedSatoshis {
        SignedSatoshis(self.0.abs())
    }

    pub fn from_btc(btc: Decimal) -> Self {
        let sats = btc * SATS_PER_BTC;
        assert!(sats.trunc() == sats, "Satoshis must be an integer");
        Self(i64::try_from(sats).expect("Satoshis must be integer"))
    }

    pub fn into_inner(self) -> i64 {
        self.0
    }
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("ConversionError - DecimalError: {0}")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("ConversionError - UnexpectedNegativeNumber: {0}")]
    UnexpectedNegativeNumber(rust_decimal::Decimal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Satoshis(u64);
async_graphql::scalar!(Satoshis);

impl fmt::Display for Satoshis {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Satoshis {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add<Satoshis> for Satoshis {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Satoshis(self.0 + other.0)
    }
}

impl std::ops::Sub<Satoshis> for Satoshis {
    type Output = Satoshis;

    fn sub(self, other: Satoshis) -> Satoshis {
        Satoshis(self.0 - other.0)
    }
}

impl From<u64> for Satoshis {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Satoshis {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_btc(self) -> Decimal {
        Decimal::from(self.0) / SATS_PER_BTC
    }

    pub fn try_from_btc(btc: Decimal) -> Result<Self, ConversionError> {
        let sats = btc * SATS_PER_BTC;
        assert!(sats.trunc() == sats, "Satoshis must be an integer");
        if sats < Decimal::new(0, 0) {
            return Err(ConversionError::UnexpectedNegativeNumber(sats));
        }
        Ok(Self(u64::try_from(sats)?))
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }
}

impl TryFrom<SignedSatoshis> for Satoshis {
    type Error = ConversionError;

    fn try_from(value: SignedSatoshis) -> Result<Self, Self::Error> {
        Self::try_from_btc(value.to_btc())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignedUsdCents(i64);
async_graphql::scalar!(SignedUsdCents);

impl SignedUsdCents {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_usd(self) -> Decimal {
        Decimal::from(self.0) / CENTS_PER_USD
    }

    pub fn from_usd(usd: Decimal) -> Self {
        let cents = usd * CENTS_PER_USD;
        assert!(cents.trunc() == cents, "Cents must be an integer");
        Self(i64::try_from(cents).expect("Cents must be integer"))
    }

    pub fn into_inner(self) -> i64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<UsdCents> for SignedUsdCents {
    fn from(cents: UsdCents) -> Self {
        Self(i64::try_from(cents.0).expect("Cents must be integer sized for i64"))
    }
}

impl fmt::Display for SignedUsdCents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Sub<SignedUsdCents> for SignedUsdCents {
    type Output = SignedUsdCents;

    fn sub(self, other: SignedUsdCents) -> SignedUsdCents {
        SignedUsdCents(self.0 - other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UsdCents(u64);
async_graphql::scalar!(UsdCents);

impl std::ops::SubAssign for UsdCents {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl std::ops::AddAssign for UsdCents {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl UsdCents {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_usd(self) -> Decimal {
        Decimal::from(self.0) / CENTS_PER_USD
    }

    pub fn try_from_usd(usd: Decimal) -> Result<Self, ConversionError> {
        let cents = usd * CENTS_PER_USD;
        assert!(cents.trunc() == cents, "Cents must be an integer");
        if cents < Decimal::new(0, 0) {
            return Err(ConversionError::UnexpectedNegativeNumber(cents));
        }
        Ok(Self(u64::try_from(cents)?))
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u64> for UsdCents {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for UsdCents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Sub<UsdCents> for UsdCents {
    type Output = UsdCents;

    fn sub(self, other: UsdCents) -> UsdCents {
        UsdCents(self.0 - other.0)
    }
}

impl std::ops::Add<UsdCents> for UsdCents {
    type Output = Self;

    fn add(self, other: UsdCents) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Mul<u64> for UsdCents {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        Self(self.0 * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PriceOfOneBTC(UsdCents);

impl PriceOfOneBTC {
    pub const ZERO: Self = Self::new(UsdCents::ZERO);

    pub const fn new(price: UsdCents) -> Self {
        Self(price)
    }

    pub fn cents_to_sats_round_up(self, cents: UsdCents) -> Satoshis {
        let btc = (cents.to_usd() / self.0.to_usd())
            .round_dp_with_strategy(8, RoundingStrategy::AwayFromZero);
        Satoshis::try_from_btc(btc).expect("Decimal should have no fractional component here")
    }

    pub fn sats_to_cents_round_down(self, sats: Satoshis) -> UsdCents {
        let usd =
            (sats.to_btc() * self.0.to_usd()).round_dp_with_strategy(2, RoundingStrategy::ToZero);
        UsdCents::try_from_usd(usd).expect("Decimal should have no fractional component here")
    }

    pub fn into_inner(self) -> UsdCents {
        self.0
    }
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportProgress {
    Running,
    Complete,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cents_to_sats_trivial() {
        let price =
            PriceOfOneBTC::new(UsdCents::try_from_usd(rust_decimal_macros::dec!(1000)).unwrap());
        let cents = UsdCents::try_from_usd(rust_decimal_macros::dec!(1000)).unwrap();
        assert_eq!(
            Satoshis::try_from_btc(dec!(1)).unwrap(),
            price.cents_to_sats_round_up(cents)
        );
    }

    #[test]
    fn cents_to_sats_complex() {
        let price =
            PriceOfOneBTC::new(UsdCents::try_from_usd(rust_decimal_macros::dec!(60000)).unwrap());
        let cents = UsdCents::try_from_usd(rust_decimal_macros::dec!(100)).unwrap();
        assert_eq!(
            Satoshis::try_from_btc(dec!(0.00166667)).unwrap(),
            price.cents_to_sats_round_up(cents)
        );
    }

    #[test]
    fn sats_to_cents_trivial() {
        let price = PriceOfOneBTC::new(UsdCents::from(5_000_000));
        let sats = Satoshis::from(10_000);
        assert_eq!(UsdCents::from(500), price.sats_to_cents_round_down(sats));
    }

    #[test]
    fn sats_to_cents_complex() {
        let price = PriceOfOneBTC::new(UsdCents::from(5_000_000));
        let sats = Satoshis::from(12_345);
        assert_eq!(UsdCents::from(617), price.sats_to_cents_round_down(sats));
    }
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum CollateralAction {
    Add,
    Remove,
}
