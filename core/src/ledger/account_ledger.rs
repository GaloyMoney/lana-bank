use crate::primitives::{Satoshis, UsdCents};

use super::cala::graphql::*;

#[derive(Debug, Clone)]
pub struct BtcAccountBalance {
    pub debit: Satoshis,
    pub credit: Satoshis,
    pub net: Satoshis,
}

impl From<general_ledger::balances> for BtcAccountBalance {
    fn from(balances: general_ledger::balances) -> Self {
        Self {
            debit: Satoshis::from_btc(balances.dr_balance.units),
            credit: Satoshis::from_btc(balances.cr_balance.units),
            net: Satoshis::from_btc(balances.normal_balance.units),
        }
    }
}

impl Default for BtcAccountBalance {
    fn default() -> Self {
        Self {
            debit: Satoshis::ZERO,
            credit: Satoshis::ZERO,
            net: Satoshis::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsdAccountBalance {
    pub debit: UsdCents,
    pub credit: UsdCents,
    pub net: UsdCents,
}

impl From<general_ledger::balances> for UsdAccountBalance {
    fn from(balances: general_ledger::balances) -> Self {
        Self {
            debit: UsdCents::from_usd(balances.dr_balance.units),
            credit: UsdCents::from_usd(balances.cr_balance.units),
            net: UsdCents::from_usd(balances.normal_balance.units),
        }
    }
}

impl Default for UsdAccountBalance {
    fn default() -> Self {
        Self {
            debit: UsdCents::ZERO,
            credit: UsdCents::ZERO,
            net: UsdCents::ZERO,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LayeredBtcAccountBalances {
    pub settled: BtcAccountBalance,
    pub pending: BtcAccountBalance,
    pub encumbrance: BtcAccountBalance,
}

impl From<general_ledger::GeneralLedgerAccountSetBtcBalances> for LayeredBtcAccountBalances {
    fn from(btc_balances_by_layer: general_ledger::GeneralLedgerAccountSetBtcBalances) -> Self {
        Self {
            settled: BtcAccountBalance::from(btc_balances_by_layer.settled),
            pending: BtcAccountBalance::from(btc_balances_by_layer.pending),
            encumbrance: BtcAccountBalance::from(btc_balances_by_layer.encumbrance),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LayeredUsdAccountBalances {
    pub settled: UsdAccountBalance,
    pub pending: UsdAccountBalance,
    pub encumbrance: UsdAccountBalance,
}

impl From<general_ledger::GeneralLedgerAccountSetUsdBalances> for LayeredUsdAccountBalances {
    fn from(usd_balances_by_layer: general_ledger::GeneralLedgerAccountSetUsdBalances) -> Self {
        Self {
            settled: UsdAccountBalance::from(usd_balances_by_layer.settled),
            pending: UsdAccountBalance::from(usd_balances_by_layer.pending),
            encumbrance: UsdAccountBalance::from(usd_balances_by_layer.encumbrance),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountBalancesByCurrency {
    pub btc: LayeredBtcAccountBalances,
    pub usd: LayeredUsdAccountBalances,
    pub usdt: LayeredUsdAccountBalances,
}

#[derive(Debug, Clone)]
pub struct AccountLedgerLineItem {
    pub name: String,
    pub total_balance: AccountBalancesByCurrency,
}

#[derive(Debug, Clone)]
pub struct AccountLedgerLineItemAndCursor {
    pub cursor: String,
    pub line_item: AccountLedgerLineItem,
}

impl From<general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccount>
    for AccountLedgerLineItem
{
    fn from(node: general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccount) -> Self {
        AccountLedgerLineItem {
            name: node.name,
            total_balance: AccountBalancesByCurrency {
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

impl From<general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccountSet>
    for AccountLedgerLineItem
{
    fn from(node: general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccountSet) -> Self {
        AccountLedgerLineItem {
            name: node.name,
            total_balance: AccountBalancesByCurrency {
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

pub struct AccountLedgerSummary {
    pub name: String,
    pub total_balance: AccountBalancesByCurrency,
    pub line_item_balances: Vec<AccountLedgerLineItemAndCursor>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl From<general_ledger::GeneralLedgerAccountSet> for AccountLedgerSummary {
    fn from(account_set: general_ledger::GeneralLedgerAccountSet) -> Self {
        let line_item_balances: Vec<AccountLedgerLineItemAndCursor> = account_set
            .members
            .edges
            .iter()
            .map(|e| match &e.node {
                general_ledger::GeneralLedgerAccountSetMembersEdgesNode::Account(node) => {
                    AccountLedgerLineItemAndCursor {
                        line_item: AccountLedgerLineItem::from(node.clone()),
                        cursor: e.cursor.clone(),
                    }
                }
                general_ledger::GeneralLedgerAccountSetMembersEdgesNode::AccountSet(node) => {
                    AccountLedgerLineItemAndCursor {
                        line_item: AccountLedgerLineItem::from(node.clone()),
                        cursor: e.cursor.clone(),
                    }
                }
            })
            .collect();

        Self {
            name: account_set.name,
            total_balance: AccountBalancesByCurrency {
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
            line_item_balances,
            has_next_page: account_set.members.page_info.has_next_page,
            has_previous_page: account_set.members.page_info.has_previous_page,
        }
    }
}
