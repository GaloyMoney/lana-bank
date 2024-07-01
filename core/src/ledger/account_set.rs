use crate::primitives::{LedgerAccountSetId, LedgerAccountSetMemberType, LedgerDebitOrCredit};

use super::{account::*, cala::graphql::*, LedgerError};

#[derive(Debug, Clone)]
pub struct LedgerAccountSetBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
}

impl From<trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccountSet>
    for LedgerAccountSetBalance
{
    fn from(node: trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetBalance {
            name: node.name,
            normal_balance_type: node.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: node.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: node.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: node.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct DebitNormalLedgerAccountSetBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: DebitNormalLedgerAccountBalancesByCurrency,
}

impl TryFrom<LedgerAccountSetBalance> for DebitNormalLedgerAccountSetBalance {
    type Error = LedgerError;

    fn try_from(balance: LedgerAccountSetBalance) -> Result<Self, LedgerError> {
        Ok(DebitNormalLedgerAccountSetBalance {
            name: balance.name,
            normal_balance_type: balance.normal_balance_type,
            balance: balance.balance.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum LedgerAccountSetMemberBalance {
    LedgerAccountBalance(LedgerAccountBalance),
    LedgerAccountSetBalance(LedgerAccountSetBalance),
}

#[derive(Debug, Clone)]
pub enum DebitNormalLedgerAccountSetMemberBalance {
    LedgerAccountBalance(DebitNormalLedgerAccountBalance),
    LedgerAccountSetBalance(DebitNormalLedgerAccountSetBalance),
}

impl TryFrom<LedgerAccountSetMemberBalance> for DebitNormalLedgerAccountSetMemberBalance {
    type Error = LedgerError;

    fn try_from(balance: LedgerAccountSetMemberBalance) -> Result<Self, LedgerError> {
        match balance {
            LedgerAccountSetMemberBalance::LedgerAccountBalance(val) => Ok(
                DebitNormalLedgerAccountSetMemberBalance::LedgerAccountBalance(
                    DebitNormalLedgerAccountBalance::try_from(val)?,
                ),
            ),
            LedgerAccountSetMemberBalance::LedgerAccountSetBalance(val) => Ok(
                DebitNormalLedgerAccountSetMemberBalance::LedgerAccountSetBalance(
                    DebitNormalLedgerAccountSetBalance::try_from(val)?,
                ),
            ),
        }
    }
}

pub struct LedgerAccountSetAndMemberBalances {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub member_balances: Vec<LedgerAccountSetMemberBalance>,
}

impl From<trial_balance::TrialBalanceAccountSet> for LedgerAccountSetAndMemberBalances {
    fn from(account_set: trial_balance::TrialBalanceAccountSet) -> Self {
        let member_balances: Vec<LedgerAccountSetMemberBalance> = account_set
            .members
            .edges
            .iter()
            .map(|e| match &e.node {
                trial_balance::TrialBalanceAccountSetMembersEdgesNode::Account(node) => {
                    LedgerAccountSetMemberBalance::LedgerAccountBalance(LedgerAccountBalance::from(
                        node.clone(),
                    ))
                }
                trial_balance::TrialBalanceAccountSetMembersEdgesNode::AccountSet(node) => {
                    LedgerAccountSetMemberBalance::LedgerAccountSetBalance(
                        LedgerAccountSetBalance::from(node.clone()),
                    )
                }
            })
            .collect();

        Self {
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: account_set.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: account_set.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: account_set.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
            member_balances,
        }
    }
}

impl From<account_set_by_id::AccountSetByIdAccountSet> for LedgerAccountSetId {
    fn from(account_set: account_set_by_id::AccountSetByIdAccountSet) -> Self {
        Self::from(account_set.account_set_id)
    }
}

impl From<LedgerAccountSetMemberType> for add_to_account_set::AccountSetMemberType {
    fn from(member_type: LedgerAccountSetMemberType) -> Self {
        match member_type {
            LedgerAccountSetMemberType::Account => Self::ACCOUNT,
            LedgerAccountSetMemberType::AccountSet => Self::ACCOUNT_SET,
        }
    }
}
