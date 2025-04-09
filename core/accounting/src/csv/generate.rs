use csv::Writer;

use crate::journal::JournalEntry;
use crate::ledger_account::LedgerAccounts;
use crate::primitives::LedgerAccountId;

use super::error::AccountingCsvError;

pub struct GenerateCsv {
    ledger_accounts: LedgerAccounts,
}

impl GenerateCsv {
    pub fn new(ledger_accounts: &LedgerAccounts) -> Self {
        Self {
            ledger_accounts: ledger_accounts.clone(),
        }
    }

    pub async fn generate_ledger_account_csv(
        &self,
        ledger_account_id: LedgerAccountId,
    ) -> Result<Vec, AccountingCsvError> {
        let history_result = self
            .ledger_accounts
            .history(sub, ledger_account_id, Default::default())
            .await;

        unimplemented!()
    }
}
