use async_graphql::*;

use lana_app::chart_of_accounts::AccountDetails;

use crate::primitives::*;

#[derive(SimpleObject)]
pub struct LedgerAccount {
    id: UUID,
    name: String,
    // code: AccountCode,
    // amounts: AccountAmountsByCurrency,
}

impl From<AccountDetails> for LedgerAccount {
    fn from(account: AccountDetails) -> Self {
        LedgerAccount {
            id: account.id.into(),
            name: account.name.to_string(),
            // code: account.code.into(),
            // amounts: account.into(),
        }
    }
}
