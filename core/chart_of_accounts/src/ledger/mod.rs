pub mod error;

use cala_ledger::{
    account::{error::AccountError, *},
    CalaLedger, Currency, DebitOrCredit, JournalId, TransactionId,
};

use crate::primitives::*;

#[derive(Clone)]
pub struct ChartOfAccountsLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    assets_account_set_id: LedgerAccountSetId,
}

impl ChartOfAccountsLedger {
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
    ) -> Result<Self, ChartOfAccountsLedgerError> {
        let assets_code = "".to_string();
        let assets_account_set_id =
            Self::create_assets_account_set(cala, journal_id, assets_code).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            assets_account_set_id,
        })
    }

    async fn create_assets_account_set(
        cala: &CalaLedger,
        journal_id: JournalId,
        code: String,
    ) -> Result<LedgerAccountSetId, ChartOfAccountsLedgerError> {
        let new_account_set = NewAccountSet::builder()
            .code(&code)
            .id(LedgerAccountSetId::new())
            .journal_id(journal_id)
            .name("Assets Account Set")
            .description("Assets Account Set for Chart of Accounts")
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Couldn't create assets account set");
        match cala.accounts().create(new_account_set).await {
            Err(AccountError::CodeAlreadyExists) => {
                let account_set = cala.account_sets().find_by_code(code).await?;
                Ok(account_set.id)
            }
            Err(e) => Err(e.into()),
            Ok(account_set) => Ok(account_set.id),
        }
    }
}
