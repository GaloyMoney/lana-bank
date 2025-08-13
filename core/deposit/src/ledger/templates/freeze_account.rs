use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{AccountId as CalaAccountId, Currency, JournalId, tx_template::Params};

pub const FREEZE_ACCOUNT_CODE: &str = "FREEZE_ACCOUNT";

#[derive(Debug)]
pub struct FreezeAccountParams {
    pub journal_id: JournalId,
    pub frozen_accounts_account_id: CalaAccountId,
    pub amount: Decimal,
    pub currency: Currency,
}

impl From<FreezeAccountParams> for Params {
    fn from(
        FreezeAccountParams {
            journal_id,
            frozen_accounts_account_id,
            amount,
            currency,
        }: FreezeAccountParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("frozen_accounts_account_id", frozen_accounts_account_id);
        params.insert("effective", crate::time::now().date_naive());
        params
    }
}
