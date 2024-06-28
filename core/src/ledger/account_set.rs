use crate::primitives::{LedgerAccountSetId, LedgerAccountSetMemberType};

use super::{account::*, cala::graphql::*};

#[derive(Debug, Clone)]
pub struct LedgerAccountSetBalance {
    pub name: String,
    pub balance: LedgerAccountBalancesByCurrency,
}

impl From<general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccountSet>
    for LedgerAccountSetBalance
{
    fn from(node: general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetBalance {
            name: node.name,
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
pub enum LedgerAccountSetMemberBalance {
    LedgerAccountBalance(LedgerAccountBalance),
    LedgerAccountSetBalance(LedgerAccountSetBalance),
}
pub struct LedgerAccountSetAndMemberBalances {
    pub name: String,
    pub balance: LedgerAccountBalancesByCurrency,
    pub member_balances: Vec<LedgerAccountSetMemberBalance>,
}

impl From<general_ledger::GeneralLedgerAccountSet> for LedgerAccountSetAndMemberBalances {
    fn from(account_set: general_ledger::GeneralLedgerAccountSet) -> Self {
        let member_balances: Vec<LedgerAccountSetMemberBalance> = account_set
            .members
            .edges
            .iter()
            .map(|e| match &e.node {
                general_ledger::GeneralLedgerAccountSetMembersEdgesNode::Account(node) => {
                    LedgerAccountSetMemberBalance::LedgerAccountBalance(LedgerAccountBalance::from(
                        node.clone(),
                    ))
                }
                general_ledger::GeneralLedgerAccountSetMembersEdgesNode::AccountSet(node) => {
                    LedgerAccountSetMemberBalance::LedgerAccountSetBalance(
                        LedgerAccountSetBalance::from(node.clone()),
                    )
                }
            })
            .collect();

        Self {
            name: account_set.name,
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
