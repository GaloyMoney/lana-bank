use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as LedgerAccountId;
use chart_of_accounts::{CoreChartOfAccounts, TransactionAccountFactory};

use crate::{
    // accounting_init::CreditFacilitiesAccountPaths,
    primitives::{LedgerTxId, Satoshis, UsdCents},
    terms::InterestPeriod,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityAccountIds {
    pub facility_account_id: LedgerAccountId,
    pub disbursed_receivable_account_id: LedgerAccountId,
    pub collateral_account_id: LedgerAccountId,
    pub interest_receivable_account_id: LedgerAccountId,
    pub interest_account_id: LedgerAccountId,
    pub fee_income_account_id: LedgerAccountId,
}

impl CreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            facility_account_id: LedgerAccountId::new(),
            disbursed_receivable_account_id: LedgerAccountId::new(),
            collateral_account_id: LedgerAccountId::new(),
            interest_receivable_account_id: LedgerAccountId::new(),
            interest_account_id: LedgerAccountId::new(),
            fee_income_account_id: LedgerAccountId::new(),
        }
    }
}

#[derive(Clone)]
pub struct CreditFacilityAccountFactories {
    pub facility: TransactionAccountFactory,
    pub facility_omnibus: TransactionAccountFactory,
    pub disbursed_receivable: TransactionAccountFactory,
    pub collateral: TransactionAccountFactory,
    pub collateral_omnibus: TransactionAccountFactory,
    pub interest_receivable: TransactionAccountFactory,
    pub interest_income: TransactionAccountFactory,
    pub fee_income: TransactionAccountFactory,
}

// impl CreditFacilityAccountFactories {
//     pub fn new(
//         chart_of_accounts: &ChartOfAccounts,
//         credit_facilities: CreditFacilitiesAccountPaths,
//     ) -> Self {
//         Self {
//             facility: chart_of_accounts.transaction_account_factory(credit_facilities.facility),
//             facility_omnibus: chart_of_accounts
//                 .transaction_account_factory(credit_facilities.facility_omnibus),
//             disbursed_receivable: chart_of_accounts
//                 .transaction_account_factory(credit_facilities.disbursed_receivable),
//             collateral: chart_of_accounts.transaction_account_factory(credit_facilities.collateral),
//             collateral_omnibus: chart_of_accounts
//                 .transaction_account_factory(credit_facilities.collateral_omnibus),
//             interest_receivable: chart_of_accounts
//                 .transaction_account_factory(credit_facilities.interest_receivable),
//             interest_income: chart_of_accounts
//                 .transaction_account_factory(credit_facilities.interest_income),
//             fee_income: chart_of_accounts.transaction_account_factory(credit_facilities.fee_income),
//         }
//     }
// }
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityLedgerBalance {
    pub facility: UsdCents,
    pub collateral: Satoshis,
    pub disbursed: UsdCents,
    pub disbursed_receivable: UsdCents,
    pub interest: UsdCents,
    pub interest_receivable: UsdCents,
}

impl CreditFacilityLedgerBalance {
    pub fn check_disbursal_amount(&self, amount: UsdCents) -> bool {
        amount < self.facility
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct CreditFacilityPaymentAmounts {
    pub interest: UsdCents,
    pub disbursal: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityCompletion {
    pub tx_id: LedgerTxId,
    pub collateral: Satoshis,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityActivation {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub debit_account_id: LedgerAccountId,
    pub facility_amount: UsdCents,
    pub structuring_fee_amount: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestIncurrence {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub period: InterestPeriod,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrual {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub accrued_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DisbursalData {
    pub amount: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub cancelled: bool,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub debit_account_id: LedgerAccountId,
}
