mod seed;

pub mod error;

use chart_of_accounts::{ChartId, ChartOfAccountCode};

use crate::chart_of_accounts::ChartOfAccounts;

use error::*;

#[derive(Clone)]
pub struct AccountingInit {
    pub chart_id: ChartId,
    pub deposits_control_sub_path: ChartOfAccountCode,
}

impl AccountingInit {
    pub async fn execute(chart_of_accounts: &ChartOfAccounts) -> Result<Self, AccountingInitError> {
        Ok(seed::execute(chart_of_accounts).await?)
    }
}
