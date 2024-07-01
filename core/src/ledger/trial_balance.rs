use super::{
    account::DebitNormalLedgerAccountBalancesByCurrency,
    account_set::DebitNormalLedgerAccountSetMemberBalance, LedgerError,
};

#[derive(Debug, Clone)]
pub struct TrialBalance {
    pub name: String,
    pub balance: DebitNormalLedgerAccountBalancesByCurrency,
    pub member_balances: Vec<DebitNormalLedgerAccountSetMemberBalance>,
}

impl TryFrom<super::account_set::LedgerAccountSetAndMemberBalances> for TrialBalance {
    type Error = LedgerError;

    fn try_from(
        account_set: super::account_set::LedgerAccountSetAndMemberBalances,
    ) -> Result<Self, LedgerError> {
        Ok(TrialBalance {
            name: account_set.name,
            balance: account_set.balance.try_into()?,
            member_balances: account_set
                .member_balances
                .iter()
                .map(|m| DebitNormalLedgerAccountSetMemberBalance::try_from(m.clone()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
