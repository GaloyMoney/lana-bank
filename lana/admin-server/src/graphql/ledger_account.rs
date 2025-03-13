use async_graphql::*;
use serde::{Deserialize, Serialize};

use lana_app::chart_of_accounts::AccountDetails;

use crate::primitives::*;

#[derive(SimpleObject)]
pub struct LedgerAccount {
    id: UUID,
    name: String,
    code: AccountCode,
    // amounts: AccountAmountsByCurrency,
}

impl From<AccountDetails> for LedgerAccount {
    fn from(account: AccountDetails) -> Self {
        LedgerAccount {
            id: account.id.into(),
            name: account.name.to_string(),
            code: AccountCode(account.code.to_string()),
            // amounts: account.into(),
        }
    }
}

scalar!(AccountCode);
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct AccountCode(String);
