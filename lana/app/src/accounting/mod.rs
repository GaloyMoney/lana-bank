mod seed;

pub mod error;

use chart_of_accounts::{ChartId, ChartOfAccountCode};

use crate::chart_of_accounts::ChartOfAccounts;

use error::*;

#[derive(Clone)]
pub struct AccountingValues {
    pub id: ChartId,
    pub deposits_control_sub_path: ChartOfAccountCode,
}

#[derive(Clone)]
pub struct Accounting {
    pub values: AccountingValues,
}

impl Accounting {
    pub async fn init(chart_of_accounts: &ChartOfAccounts) -> Result<Self, AccountingError> {
        let values = seed::execute(chart_of_accounts).await?;
        Ok(Accounting { values })
    }
}
