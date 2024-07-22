use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::fmt;

crate::entity_id! { UserId }
crate::entity_id! { LineOfCreditContractId }
crate::entity_id! { WithdrawId }
crate::entity_id! { JobId }
crate::entity_id! { LoanId }
crate::entity_id! { LoanTermsId }

impl From<LoanId> for JobId {
    fn from(id: LoanId) -> Self {
        JobId::from(id.0)
    }
}

// Consider importing from cala
#[derive(Debug)]
pub enum LedgerAccountSetMemberType {
    Account,
    AccountSet,
}

crate::entity_id! { BfxIntegrationId }

#[derive(Debug)]
pub enum BfxAddressType {
    Bitcoin,
    Tron,
}

#[derive(Debug, Deserialize, Clone, Copy, Serialize)]
pub enum KycLevel {
    NotKyced,
    Basic,
    Advanced,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LoanStatus {
    New,
    Active,
    Closed,
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

#[derive(Debug)]
pub enum AccountStatus {
    Active,
    Inactive,
}

pub enum BfxWithdrawalMethod {
    Bitcoin,
    TronUsdt,
}

pub enum DebitOrCredit {
    Debit,
    Credit,
}

pub use cala_types::primitives::{
    AccountId as LedgerAccountId, AccountSetId as LedgerAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};

pub const SATS_PER_BTC: Decimal = dec!(100_000_000);
pub const CENTS_PER_USD: Decimal = dec!(100);

#[derive(Debug, Clone)]
pub struct Subject(String);

impl From<String> for Subject {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Subject {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Subject {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Group(String);

impl From<String> for Group {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Group {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Group {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignedSatoshis(i64);

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

impl SignedSatoshis {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_btc(self) -> Decimal {
        Decimal::from(self.0) / SATS_PER_BTC
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Satoshis(u64);

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

impl std::ops::Sub<Satoshis> for Satoshis {
    type Output = Satoshis;

    fn sub(self, other: Satoshis) -> Satoshis {
        Satoshis(self.0 - other.0)
    }
}

impl Satoshis {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_btc(self) -> Decimal {
        Decimal::from(self.0) / SATS_PER_BTC
    }

    pub fn from_btc(btc: Decimal) -> Self {
        let sats = btc * SATS_PER_BTC;
        assert!(sats.trunc() == sats, "Satoshis must be an integer");
        Self(u64::try_from(sats).expect("Satoshis must be positive integer"))
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignedUsdCents(i64);

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

impl UsdCents {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_usd(self) -> Decimal {
        Decimal::from(self.0) / CENTS_PER_USD
    }

    pub fn from_usd(usd: Decimal) -> Self {
        let cents = usd * CENTS_PER_USD;
        assert!(cents.trunc() == cents, "Cents must be an integer");
        Self(u64::try_from(cents).expect("Cents must be positive integer"))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PriceOfOneBTC(UsdCents);

impl PriceOfOneBTC {
    pub const fn new(price: UsdCents) -> Self {
        Self(price)
    }

    pub fn cents_to_sats(self, cents: UsdCents, rounding_strategy: RoundingStrategy) -> Satoshis {
        let btc = (cents.to_usd() / self.0.to_usd()).round_dp_with_strategy(8, rounding_strategy);
        Satoshis::from_btc(btc)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cents_to_sats_trivial() {
        let price = PriceOfOneBTC::new(UsdCents::from_usd(rust_decimal_macros::dec!(1000)));
        let cents = UsdCents::from_usd(rust_decimal_macros::dec!(1000));
        assert_eq!(
            Satoshis::from_btc(dec!(1)),
            price.cents_to_sats(cents, rust_decimal::RoundingStrategy::AwayFromZero)
        );
    }

    #[test]
    fn cents_to_sats_complex() {
        let price = PriceOfOneBTC::new(UsdCents::from_usd(rust_decimal_macros::dec!(60000)));
        let cents = UsdCents::from_usd(rust_decimal_macros::dec!(100));
        assert_eq!(
            Satoshis::from_btc(dec!(0.00166667)),
            price.cents_to_sats(cents, rust_decimal::RoundingStrategy::AwayFromZero)
        );
    }
}
