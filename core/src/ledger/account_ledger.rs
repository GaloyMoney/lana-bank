use crate::primitives::{Satoshis, UsdCents};

use super::cala::graphql::*;

struct BtcAccountBalance {
    debit: Satoshis,
    credit: Satoshis,
    net: Satoshis,
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

struct UsdAccountBalance {
    debit: UsdCents,
    credit: UsdCents,
    net: UsdCents,
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

struct LayeredBtcAccountBalances {
    settled: BtcAccountBalance,
    pending: BtcAccountBalance,
    encumbrance: BtcAccountBalance,
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

impl Default for LayeredBtcAccountBalances {
    fn default() -> Self {
        Self {
            settled: BtcAccountBalance::default(),
            pending: BtcAccountBalance::default(),
            encumbrance: BtcAccountBalance::default(),
        }
    }
}

struct LayeredUsdAccountBalances {
    settled: UsdAccountBalance,
    pending: UsdAccountBalance,
    encumbrance: UsdAccountBalance,
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

impl Default for LayeredUsdAccountBalances {
    fn default() -> Self {
        Self {
            settled: UsdAccountBalance::default(),
            pending: UsdAccountBalance::default(),
            encumbrance: UsdAccountBalance::default(),
        }
    }
}

struct AccountBalancesByCurrency {
    btc: LayeredBtcAccountBalances,
    usd: LayeredUsdAccountBalances,
    usdt: LayeredUsdAccountBalances,
}

pub struct AccountLedgerSummary {
    name: String,
    total_balance: AccountBalancesByCurrency,
}

impl From<general_ledger::GeneralLedgerAccountSet> for AccountLedgerSummary {
    fn from(account_set: general_ledger::GeneralLedgerAccountSet) -> Self {
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
        }
    }
}
