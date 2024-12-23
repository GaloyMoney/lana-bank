mod constants;
mod seed;

pub mod error;

use chart_of_accounts::{ChartId, ChartOfAccountCode};

use crate::{app::primitives::*, chart_of_accounts::ChartOfAccounts};

use cala_ledger::{CalaLedger, JournalId};

use error::*;

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
