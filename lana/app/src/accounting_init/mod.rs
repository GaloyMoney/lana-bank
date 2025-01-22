pub mod constants;
mod primitives;
mod seed;

pub mod error;

use chart_of_accounts::ChartId;

use crate::{chart_of_accounts::ChartOfAccounts, statements::Statements};

use cala_ledger::CalaLedger;

use error::*;
pub use primitives::CreditFacilitiesAccountPaths;
use primitives::{ChartIds, DepositsAccountPaths, LedgerJournalId, TrialBalanceStatementIds};

#[derive(Clone)]
pub struct JournalInit {
    pub journal_id: LedgerJournalId,
}

impl JournalInit {
    pub async fn journal(cala: &CalaLedger) -> Result<Self, AccountingInitError> {
        seed::journal::init(cala).await
    }
}

#[derive(Clone)]
pub struct StatementsInit {
    trial_balance_ids: TrialBalanceStatementIds,
}

impl StatementsInit {
    pub async fn statements(statements: &Statements) -> Result<Self, AccountingInitError> {
        seed::statements::init(statements).await
    }
}

#[derive(Clone)]
pub struct ChartsInit {
    pub chart_ids: ChartIds,
    pub deposits: DepositsAccountPaths,
    pub credit_facilities: CreditFacilitiesAccountPaths,
}

impl ChartsInit {
    pub async fn charts_of_accounts(
        statements: &Statements,
        chart_of_accounts: &ChartOfAccounts,
    ) -> Result<Self, AccountingInitError> {
        seed::charts_of_accounts::init(statements, chart_of_accounts).await
    }
}
