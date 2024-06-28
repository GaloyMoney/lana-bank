use crate::primitives::{LedgerAccountId, LedgerAccountSetId, Satoshis, UsdCents};

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
pub struct LedgerAccountBalancesByCurrency {
    pub btc: LayeredBtcAccountBalances,
    pub usd: LayeredUsdAccountBalances,
    pub usdt: LayeredUsdAccountBalances,
}

pub struct LedgerAccount {
    pub id: LedgerAccountId,
    pub settled_btc_balance: Satoshis,
    pub settled_usd_balance: UsdCents,
    pub account_set_ids: Vec<LedgerAccountSetId>,
}

#[derive(Debug, Clone)]
pub struct LedgerAccountBalance {
    pub name: String,
    pub balance: LedgerAccountBalancesByCurrency,
}

impl From<general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccount>
    for LedgerAccountBalance
{
    fn from(node: general_ledger::GeneralLedgerAccountSetMembersEdgesNodeOnAccount) -> Self {
        LedgerAccountBalance {
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

impl From<account_by_external_id::AccountByExternalIdAccountByExternalId> for LedgerAccount {
    fn from(account: account_by_external_id::AccountByExternalIdAccountByExternalId) -> Self {
        LedgerAccount {
            id: LedgerAccountId::from(account.account_id),
            settled_usd_balance: account
                .usd_balance
                .map(|b| UsdCents::from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| UsdCents::ZERO),
            settled_btc_balance: account
                .btc_balance
                .map(|b| Satoshis::from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Satoshis::ZERO),
            account_set_ids: account
                .sets
                .edges
                .iter()
                .map(|e| e.node.account_set_id)
                .map(LedgerAccountSetId::from)
                .collect(),
        }
    }
}

impl From<account_by_id::AccountByIdAccount> for LedgerAccount {
    fn from(account: account_by_id::AccountByIdAccount) -> Self {
        LedgerAccount {
            id: LedgerAccountId::from(account.account_id),
            settled_usd_balance: account
                .usd_balance
                .map(|b| UsdCents::from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| UsdCents::ZERO),
            settled_btc_balance: account
                .btc_balance
                .map(|b| Satoshis::from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Satoshis::ZERO),
            account_set_ids: account
                .sets
                .edges
                .iter()
                .map(|e| e.node.account_set_id)
                .map(LedgerAccountSetId::from)
                .collect(),
        }
    }
}
