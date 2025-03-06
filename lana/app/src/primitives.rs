use serde::{Deserialize, Serialize};

use std::fmt;

pub use chart_of_accounts::{new::AltChartId, ChartId};
pub use core_credit::{
    CollateralAction, CreditFacilityId, CreditFacilityStatus, DisbursalId, DisbursalIdx,
    DisbursalStatus, PaymentId,
};
pub use core_customer::CustomerId;
pub use core_money::*;
pub use core_price::PriceOfOneBTC;
pub use core_user::UserId;
pub use deposit::{DepositAccountHolderId, DepositAccountId, DepositId, WithdrawalId};
pub use governance::{ApprovalProcessId, CommitteeId, CommitteeMemberId, PolicyId};
pub use job::JobId;
pub use lana_ids::*;
pub use rbac_types::{LanaRole, Role, Subject};

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

#[derive(async_graphql::Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoanStatus {
    New,
    Active,
    Closed,
}

pub use cala_ledger::primitives::{
    AccountId as LedgerAccountId, AccountSetId as LedgerAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};

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
