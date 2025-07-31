pub use core_access::{PermissionSetId, RoleId, UserId};
pub use core_accounting::{
    AccountSpec, BalanceRange, Chart, ChartId, LedgerTransactionId, ManualTransactionId,
};
pub use core_credit::{
    CollateralAction, CollateralId, CreditFacilityId, CreditFacilityStatus, DisbursalId,
    DisbursalStatus, PaymentAllocationId, PaymentId, TermsTemplateId,
};
pub use core_custody::{CustodianId, WalletId};
pub use core_customer::{CustomerDocumentId, CustomerId};
pub use core_deposit::{DepositAccountHolderId, DepositAccountId, DepositId, WithdrawalId};
pub use core_money::*;
pub use core_price::PriceOfOneBTC;
pub use core_report::ReportId;
pub use document_storage::{DocumentId, ReferenceId};
pub use governance::{ApprovalProcessId, CommitteeId, CommitteeMemberId, PolicyId};
pub use job::JobId;
pub use lana_ids::*;
pub use rbac_types::Subject;

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, Currency,
    DebitOrCredit as CalaDebitOrCredit, EntryId as CalaEntryId, JournalId as CalaJournalId,
    TransactionId as CalaTxId, TxTemplateId as CalaTxTemplateId,
};

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn cents_to_sats_trivial() {
        let price = PriceOfOneBTC::new(UsdCents::try_from_usd(dec!(1000)).unwrap());
        let cents = UsdCents::try_from_usd(dec!(1000)).unwrap();
        assert_eq!(
            Satoshis::try_from_btc(dec!(1)).unwrap(),
            price.cents_to_sats_round_up(cents)
        );
    }

    #[test]
    fn cents_to_sats_complex() {
        let price = PriceOfOneBTC::new(UsdCents::try_from_usd(dec!(60000)).unwrap());
        let cents = UsdCents::try_from_usd(dec!(100)).unwrap();
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
