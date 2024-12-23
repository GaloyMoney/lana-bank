mod constants;
mod seed;

pub mod error;

use chart_of_accounts::{ChartId, ChartOfAccountCode};

use crate::chart_of_accounts::ChartOfAccounts;

use cala_ledger::{CalaLedger, JournalId};

use error::*;

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone)]
pub struct DepositsAccountPaths {
    pub deposits: ChartOfAccountCode,
}

#[derive(Clone)]
pub struct CreditFacilitiesAccountPaths {
    pub collateral: ChartOfAccountCode,
    pub facility: ChartOfAccountCode,
    pub disbursed_receivable: ChartOfAccountCode,
    pub interest_receivable: ChartOfAccountCode,
    pub interest_income: ChartOfAccountCode,
    pub fee_income: ChartOfAccountCode,
}

#[derive(Clone)]
pub struct AccountingInit {
    pub journal_id: JournalId,
    pub chart_ids: ChartIds,
    pub deposits: DepositsAccountPaths,
    pub credit_facilities: CreditFacilitiesAccountPaths,
}

impl AccountingInit {
    pub async fn execute(
        cala: &CalaLedger,
        chart_of_accounts: &ChartOfAccounts,
    ) -> Result<Self, AccountingInitError> {
        seed::execute(cala, chart_of_accounts).await
    }
}
