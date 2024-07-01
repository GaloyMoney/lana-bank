use async_graphql::*;

use crate::server::shared_graphql::primitives::{Satoshis, UsdCents};

#[derive(SimpleObject)]
struct BtcAccountBalance {
    debit: Satoshis,
    credit: Satoshis,
    net_debit: Satoshis,
}

impl From<crate::ledger::account::BtcAccountBalance> for BtcAccountBalance {
    fn from(balance: crate::ledger::account::BtcAccountBalance) -> Self {
        BtcAccountBalance {
            debit: balance.debit.into(),
            credit: balance.credit.into(),
            net_debit: balance.net.into(), // FIXME
        }
    }
}

impl From<crate::ledger::account::DebitNormalBtcAccountBalance> for BtcAccountBalance {
    fn from(balance: crate::ledger::account::DebitNormalBtcAccountBalance) -> Self {
        BtcAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net_debit: balance.net_debit,
        }
    }
}

#[derive(SimpleObject)]
struct UsdAccountBalance {
    debit: UsdCents,
    credit: UsdCents,
    net_debit: UsdCents,
}

impl From<crate::ledger::account::UsdAccountBalance> for UsdAccountBalance {
    fn from(balance: crate::ledger::account::UsdAccountBalance) -> Self {
        UsdAccountBalance {
            debit: balance.debit.into(),
            credit: balance.credit.into(),
            net_debit: balance.net.into(), // FIXME
        }
    }
}

impl From<crate::ledger::account::DebitNormalUsdAccountBalance> for UsdAccountBalance {
    fn from(balance: crate::ledger::account::DebitNormalUsdAccountBalance) -> Self {
        UsdAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net_debit: balance.net_debit,
        }
    }
}

#[derive(SimpleObject)]
struct LayeredBtcAccountBalances {
    all: BtcAccountBalance,
    settled: BtcAccountBalance,
    pending: BtcAccountBalance,
    encumbrance: BtcAccountBalance,
}

impl From<crate::ledger::account::LayeredBtcAccountBalances> for LayeredBtcAccountBalances {
    fn from(balances: crate::ledger::account::LayeredBtcAccountBalances) -> Self {
        LayeredBtcAccountBalances {
            all: balances.all_layers.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

impl From<crate::ledger::account::DebitNormalLayeredBtcAccountBalances>
    for LayeredBtcAccountBalances
{
    fn from(balances: crate::ledger::account::DebitNormalLayeredBtcAccountBalances) -> Self {
        LayeredBtcAccountBalances {
            all: balances.all_layers.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct LayeredUsdAccountBalances {
    all: UsdAccountBalance,
    settled: UsdAccountBalance,
    pending: UsdAccountBalance,
    encumbrance: UsdAccountBalance,
}

impl From<crate::ledger::account::LayeredUsdAccountBalances> for LayeredUsdAccountBalances {
    fn from(balances: crate::ledger::account::LayeredUsdAccountBalances) -> Self {
        LayeredUsdAccountBalances {
            all: balances.all_layers.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

impl From<crate::ledger::account::DebitNormalLayeredUsdAccountBalances>
    for LayeredUsdAccountBalances
{
    fn from(balances: crate::ledger::account::DebitNormalLayeredUsdAccountBalances) -> Self {
        LayeredUsdAccountBalances {
            all: balances.all_layers.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountBalancesByCurrency {
    btc: LayeredBtcAccountBalances,
    usd: LayeredUsdAccountBalances,
    usdt: LayeredUsdAccountBalances,
}

impl From<crate::ledger::account::DebitNormalLedgerAccountBalancesByCurrency>
    for AccountBalancesByCurrency
{
    fn from(balances: crate::ledger::account::DebitNormalLedgerAccountBalancesByCurrency) -> Self {
        AccountBalancesByCurrency {
            btc: balances.btc.into(),
            usd: balances.usd.into(),
            usdt: balances.usdt.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountBalance {
    pub name: String,
    pub balance: AccountBalancesByCurrency,
}

impl From<crate::ledger::account::DebitNormalLedgerAccountBalance> for AccountBalance {
    fn from(account_balance: crate::ledger::account::DebitNormalLedgerAccountBalance) -> Self {
        AccountBalance {
            name: account_balance.name,
            balance: account_balance.balance.into(),
        }
    }
}
