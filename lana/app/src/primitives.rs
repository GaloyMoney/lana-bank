use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};

use std::fmt;

pub use core_money::*;
pub use core_user::UserId;
pub use deposit::{DepositAccountId, DepositId, WithdrawalId};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
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

#[derive(
    async_graphql::Enum,
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
pub enum CreditFacilityStatus {
    #[default]
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

#[derive(
    async_graphql::Enum,
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
pub enum AccountStatus {
    #[default]
    Inactive,
    Active,
}

pub use cala_ledger::primitives::{
    AccountId as LedgerAccountId, AccountSetId as LedgerAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};

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
    use rust_decimal_macros::dec;

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
