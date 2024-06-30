use async_graphql::*;

use super::account::AccountBalancesByCurrency;

#[derive(SimpleObject)]
pub struct NetDebitAccountSetBalance {
    name: String,
    balance: AccountBalancesByCurrency,
}

impl From<crate::ledger::account_set::DebitNormalLedgerAccountSetBalance>
    for NetDebitAccountSetBalance
{
    fn from(line_item: crate::ledger::account_set::DebitNormalLedgerAccountSetBalance) -> Self {
        NetDebitAccountSetBalance {
            name: line_item.name,
            balance: line_item.balance.into(),
        }
    }
}

#[derive(Union)]
enum NetDebitAccountSetMemberBalance {
    Account(super::account::NetDebitAccountBalance),
    AccountSet(NetDebitAccountSetBalance),
}

impl From<crate::ledger::account_set::DebitNormalLedgerAccountSetMemberBalance>
    for NetDebitAccountSetMemberBalance
{
    fn from(
        member_balance: crate::ledger::account_set::DebitNormalLedgerAccountSetMemberBalance,
    ) -> Self {
        match member_balance {
            crate::ledger::account_set::DebitNormalLedgerAccountSetMemberBalance::LedgerAccountBalance(
                val,
            ) => NetDebitAccountSetMemberBalance::Account(val.into()),
            crate::ledger::account_set::DebitNormalLedgerAccountSetMemberBalance::LedgerAccountSetBalance(
                val,
            ) => NetDebitAccountSetMemberBalance::AccountSet(val.into()),
        }
    }
}

#[derive(SimpleObject)]
pub struct TrialBalance {
    name: String,
    balance: AccountBalancesByCurrency,
    member_balances: Vec<NetDebitAccountSetMemberBalance>,
}

impl From<crate::ledger::trial_balance::TrialBalance> for TrialBalance {
    fn from(account_ledger: crate::ledger::trial_balance::TrialBalance) -> Self {
        TrialBalance {
            name: account_ledger.name,
            balance: account_ledger.balance.into(),
            member_balances: account_ledger
                .member_balances
                .iter()
                .map(|l| NetDebitAccountSetMemberBalance::from(l.clone()))
                .collect(),
        }
    }
}
