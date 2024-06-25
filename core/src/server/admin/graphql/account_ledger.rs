use async_graphql::*;

use crate::server::shared_graphql::primitives::{Satoshis, UsdCents};

#[derive(SimpleObject)]
struct BtcAccountBalance {
    debit: Satoshis,
    credit: Satoshis,
    net: Satoshis,
}

impl From<crate::ledger::account_ledger::BtcAccountBalance> for BtcAccountBalance {
    fn from(balance: crate::ledger::account_ledger::BtcAccountBalance) -> Self {
        BtcAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net: balance.net,
        }
    }
}

#[derive(SimpleObject)]
struct UsdAccountBalance {
    debit: UsdCents,
    credit: UsdCents,
    net: UsdCents,
}

impl From<crate::ledger::account_ledger::UsdAccountBalance> for UsdAccountBalance {
    fn from(balance: crate::ledger::account_ledger::UsdAccountBalance) -> Self {
        UsdAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net: balance.net,
        }
    }
}

#[derive(SimpleObject)]
struct LayeredBtcAccountBalances {
    settled: BtcAccountBalance,
    pending: BtcAccountBalance,
    encumbrance: BtcAccountBalance,
}

impl From<crate::ledger::account_ledger::LayeredBtcAccountBalances> for LayeredBtcAccountBalances {
    fn from(balances: crate::ledger::account_ledger::LayeredBtcAccountBalances) -> Self {
        LayeredBtcAccountBalances {
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct LayeredUsdAccountBalances {
    settled: UsdAccountBalance,
    pending: UsdAccountBalance,
    encumbrance: UsdAccountBalance,
}

impl From<crate::ledger::account_ledger::LayeredUsdAccountBalances> for LayeredUsdAccountBalances {
    fn from(balances: crate::ledger::account_ledger::LayeredUsdAccountBalances) -> Self {
        LayeredUsdAccountBalances {
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct AccountBalancesByCurrency {
    btc: LayeredBtcAccountBalances,
    usd: LayeredUsdAccountBalances,
    usdt: LayeredUsdAccountBalances,
}

impl From<crate::ledger::account_ledger::AccountBalancesByCurrency> for AccountBalancesByCurrency {
    fn from(balances: crate::ledger::account_ledger::AccountBalancesByCurrency) -> Self {
        AccountBalancesByCurrency {
            btc: balances.btc.into(),
            usd: balances.usd.into(),
            usdt: balances.usdt.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountLedgerLineItem {
    name: String,
    total_balance: AccountBalancesByCurrency,
}

#[derive(SimpleObject)]
pub struct AccountLedgerSummary {
    name: String,
    total_balance: AccountBalancesByCurrency,
    line_item_balances: Vec<AccountLedgerLineItem>,
}

impl From<crate::ledger::account_ledger::AccountLedgerSummary> for AccountLedgerSummary {
    fn from(account_ledger: crate::ledger::account_ledger::AccountLedgerSummary) -> Self {
        AccountLedgerSummary {
            name: account_ledger.name,
            total_balance: account_ledger.total_balance.into(),
            line_item_balances: Vec::new(),
        }
    }
}
