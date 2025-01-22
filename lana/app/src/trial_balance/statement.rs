use crate::primitives::{Currency, LedgerAccountSetId};

use super::ledger::{AccountBalance, LedgerAccountSetDetails, LedgerAccountSetDetailsWithAccounts};

pub struct StatementAccountSet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub balance: StatementAccountSetBalance,
}

impl From<LedgerAccountSetDetails> for StatementAccountSet {
    fn from(details: LedgerAccountSetDetails) -> Self {
        Self {
            id: details.values.id,
            name: details.values.name,
            description: details.values.description,
            balance: details.balance.into(),
        }
    }
}

pub struct StatementAccountSetWithAccounts {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub balance: StatementAccountSetBalance,
    pub accounts: Vec<StatementAccountSet>,
}

impl From<LedgerAccountSetDetailsWithAccounts> for StatementAccountSetWithAccounts {
    fn from(details: LedgerAccountSetDetailsWithAccounts) -> Self {
        Self {
            id: details.values.id,
            name: details.values.name,
            description: details.values.description,
            balance: details.balance.into(),
            accounts: details
                .accounts
                .into_iter()
                .map(StatementAccountSet::from)
                .collect(),
        }
    }
}

pub struct StatementAccountSetBalance {
    currency: Currency,
    settled: StatementBalanceAmount,
    pending: StatementBalanceAmount,
    encumbrance: StatementBalanceAmount,
}

impl From<AccountBalance> for StatementAccountSetBalance {
    fn from(balance: AccountBalance) -> Self {
        Self {
            currency: balance.details.currency,
            settled: StatementBalanceAmount {
                normal_balance: balance.settled().to_string(),
                dr_balance: balance.details.settled.dr_balance.to_string(),
                cr_balance: balance.details.settled.cr_balance.to_string(),
            },
            pending: StatementBalanceAmount {
                normal_balance: balance.pending().to_string(),
                dr_balance: balance.details.pending.dr_balance.to_string(),
                cr_balance: balance.details.pending.cr_balance.to_string(),
            },
            encumbrance: StatementBalanceAmount {
                normal_balance: balance.encumbrance().to_string(),
                dr_balance: balance.details.encumbrance.dr_balance.to_string(),
                cr_balance: balance.details.encumbrance.cr_balance.to_string(),
            },
        }
    }
}

pub struct StatementBalanceAmount {
    pub normal_balance: String, // TODO: change?
    pub dr_balance: String,     // TODO: change?
    pub cr_balance: String,     // TODO: change?
}
