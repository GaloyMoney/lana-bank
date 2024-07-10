use serde::{Deserialize, Serialize};

use super::account_set::PaginatedLedgerChartOfAccountsCategorySubAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAccountCursor {
    pub value: String,
}

impl From<&PaginatedLedgerChartOfAccountsCategorySubAccount> for SubAccountCursor {
    fn from(paginated_sub_account: &PaginatedLedgerChartOfAccountsCategorySubAccount) -> Self {
        Self {
            value: paginated_sub_account.cursor.clone(),
        }
    }
}
