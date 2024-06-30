use crate::primitives::LedgerDebitOrCredit;

use super::{account::LedgerAccountBalancesByCurrency, account_set::LedgerAccountSetMemberBalance};

#[derive(Debug, Clone)]
pub struct TrialBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub member_balances: Vec<LedgerAccountSetMemberBalance>,
}

impl From<super::account_set::LedgerAccountSetAndMemberBalances> for TrialBalance {
    fn from(account_set: super::account_set::LedgerAccountSetAndMemberBalances) -> Self {
        TrialBalance {
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type,
            balance: account_set.balance,
            member_balances: match account_set.normal_balance_type {
                LedgerDebitOrCredit::Debit => account_set
                    .member_balances
                    .iter()
                    .map(LedgerAccountSetMemberBalance::as_debit_normal)
                    .collect(),
                LedgerDebitOrCredit::Credit => account_set
                    .member_balances
                    .iter()
                    .map(LedgerAccountSetMemberBalance::as_credit_normal)
                    .collect(),
            },
        }
    }
}
