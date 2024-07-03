use async_graphql::*;

use crate::server::shared_graphql::primitives::{
    Satoshis, SignedSatoshis, SignedUsdCents, UsdCents,
};

#[derive(SimpleObject)]
struct BtcAccountBalance {
    debit: Satoshis,
    credit: Satoshis,
    net_debit: SignedSatoshis,
}

impl From<crate::ledger::account::BtcAccountBalance> for BtcAccountBalance {
    fn from(balance: crate::ledger::account::BtcAccountBalance) -> Self {
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
    net_debit: SignedUsdCents,
}

impl From<crate::ledger::account::UsdAccountBalance> for UsdAccountBalance {
    fn from(balance: crate::ledger::account::UsdAccountBalance) -> Self {
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

#[derive(SimpleObject)]
pub struct AccountBalancesByCurrency {
    btc: LayeredBtcAccountBalances,
    usd: LayeredUsdAccountBalances,
    usdt: LayeredUsdAccountBalances,
}

impl From<crate::ledger::account::LedgerAccountBalancesByCurrency> for AccountBalancesByCurrency {
    fn from(balances: crate::ledger::account::LedgerAccountBalancesByCurrency) -> Self {
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

impl From<crate::ledger::account::LedgerAccountBalance> for AccountBalance {
    fn from(account_balance: crate::ledger::account::LedgerAccountBalance) -> Self {
        AccountBalance {
            name: account_balance.name,
            balance: account_balance.balance.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountDetails {
    pub name: String,
}

impl From<crate::ledger::account::LedgerChartOfAccountsAccount> for AccountDetails {
    fn from(account_set: crate::ledger::account::LedgerChartOfAccountsAccount) -> Self {
        AccountDetails {
            name: account_set.name,
        }
    }
}
