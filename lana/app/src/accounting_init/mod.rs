mod seed;

pub mod error;

use chart_of_accounts::{ChartId, ChartOfAccountCode};

use crate::chart_of_accounts::ChartOfAccounts;

use cala_ledger::{CalaLedger, JournalId};

use error::*;

#[derive(Clone)]
pub struct AccountingInit {
    pub journal_id: JournalId,
    pub chart_id: ChartId,
    pub deposits_control_sub_path: ChartOfAccountCode,
}

impl AccountingInit {
    pub async fn execute(
        cala: &CalaLedger,
        chart_of_accounts: &ChartOfAccounts,
    ) -> Result<Self, AccountingInitError> {
        seed::execute(cala, chart_of_accounts).await
    }
}
