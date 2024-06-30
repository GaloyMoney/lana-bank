use super::{
    account::DebitNormalLedgerAccountBalancesByCurrency,
    account_set::DebitNormalLedgerAccountSetMemberBalance,
};

#[derive(Debug, Clone)]
pub struct TrialBalance {
    pub name: String,
    pub balance: DebitNormalLedgerAccountBalancesByCurrency,
    pub member_balances: Vec<DebitNormalLedgerAccountSetMemberBalance>,
}

impl From<super::account_set::LedgerAccountSetAndMemberBalances> for TrialBalance {
    fn from(account_set: super::account_set::LedgerAccountSetAndMemberBalances) -> Self {
        TrialBalance {
            name: account_set.name,
            balance: account_set.balance.into(),
            member_balances: account_set
                .member_balances
                .iter()
                .map(|m| DebitNormalLedgerAccountSetMemberBalance::from(m.clone()))
                .collect(),
        }
    }
}
