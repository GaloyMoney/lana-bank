use crate::primitives::LedgerAccountSetId;

use super::balance::{BtcStatementAccountSetBalanceRange, UsdStatementAccountSetBalanceRange};

#[derive(Clone)]
pub struct StatementAccountSetDetails {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone)]
pub struct StatementAccountSet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
}

#[derive(Clone)]
pub struct StatementAccountSetWithAccounts {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub accounts: Vec<StatementAccountSet>,
}
