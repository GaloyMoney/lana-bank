use serde::{Deserialize, Serialize};

use super::account_set::PaginatedLedgerAccountSetSubAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAccountCursor {
    pub value: String,
}

impl From<&PaginatedLedgerAccountSetSubAccount> for SubAccountCursor {
    fn from(paginated_sub_account: &PaginatedLedgerAccountSetSubAccount) -> Self {
        Self {
            value: paginated_sub_account.cursor.clone(),
        }
    }
}
