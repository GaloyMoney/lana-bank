use super::{account::LedgerAccountBalancesByCurrency, account_set::LedgerAccountSetMemberBalance};

#[derive(Debug, Clone)]
pub struct TrialBalance {
    pub name: String,
    pub balance: LedgerAccountBalancesByCurrency,
    pub member_balances: Vec<LedgerAccountSetMemberBalance>,
}

impl From<super::account_set::LedgerAccountSetAndMemberBalances> for TrialBalance {
    fn from(account_set: super::account_set::LedgerAccountSetAndMemberBalances) -> Self {
        TrialBalance {
            name: account_set.name,
            balance: account_set.balance,
            member_balances: account_set.member_balances,
        }
    }
}
