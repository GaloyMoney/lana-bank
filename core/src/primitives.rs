use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use std::fmt;

crate::entity_id! { UserId }
crate::entity_id! { FixedTermLoanId }
crate::entity_id! { LineOfCreditContractId }
crate::entity_id! { WithdrawId }
crate::entity_id! { JobId }
crate::entity_id! { LoanId}
crate::entity_id! { LoanTermsId}

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

pub enum BfxWithdrawalMethod {
    Bitcoin,
    TronUsdt,
}

impl From<FixedTermLoanId> for LedgerAccountId {
    fn from(id: FixedTermLoanId) -> Self {
        LedgerAccountId::from(id.0)
    }
}
impl From<FixedTermLoanId> for JobId {
    fn from(id: FixedTermLoanId) -> Self {
        JobId::from(id.0)
    }
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

impl Satoshis {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn to_btc(self) -> Decimal {
        Decimal::from(self.0) / SATS_PER_BTC
    }

    pub fn from_btc(btc: Decimal) -> Self {
        let sats = btc * SATS_PER_BTC;
        assert!(sats.trunc() == sats, "Satoshis must be an integer");
        Self(u64::try_from(sats).expect("Satoshis must be a positive integer"))
    }

    pub fn into_inner(self) -> u64 {
        self.0
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
        Self(u64::try_from(cents).expect("Cents must be a positive integer"))
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
    type Output = Self;

    fn sub(self, other: UsdCents) -> Self {
        assert!(self.0 >= other.0, "Subtraction result cannot be negative");
        Self(self.0 - other.0)
    }
}

pub const DUMMY_BTC_PRICE: BtcPrice = BtcPrice(6000000);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BtcPrice(u64);

impl BtcPrice {
    pub fn into_inner(self) -> u64 {
        self.0
    }
}
