use async_graphql::*;

use crate::server::shared_graphql::primitives::{
    Satoshis, SignedSatoshis, SignedUsdCents, UsdCents, UUID,
};

#[derive(SimpleObject)]
struct BtcAccountBalance {
    debit: Satoshis,
    credit: Satoshis,
    net_debit: SignedSatoshis,
    net_credit: SignedSatoshis,
}

impl From<crate::ledger::account::BtcAccountBalance> for BtcAccountBalance {
    fn from(balance: crate::ledger::account::BtcAccountBalance) -> Self {
        BtcAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net_debit: balance.net_debit,
            net_credit: balance.net_credit,
        }
    }
}

#[derive(SimpleObject)]
struct UsdAccountBalance {
    debit: UsdCents,
    credit: UsdCents,
    net_debit: SignedUsdCents,
    net_credit: SignedUsdCents,
}

impl From<crate::ledger::account::UsdAccountBalance> for UsdAccountBalance {
    fn from(balance: crate::ledger::account::UsdAccountBalance) -> Self {
        UsdAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net_debit: balance.net_debit,
            net_credit: balance.net_credit,
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
pub struct RangedBtcAccountBalances {
    start: LayeredBtcAccountBalances,
    end: LayeredBtcAccountBalances,
    diff: LayeredBtcAccountBalances,
}

impl From<crate::ledger::account::RangedBtcAccountBalances> for RangedBtcAccountBalances {
    fn from(balances: crate::ledger::account::RangedBtcAccountBalances) -> Self {
        RangedBtcAccountBalances {
            start: balances.start.into(),
            end: balances.end.into(),
            diff: balances.diff.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct RangedUsdAccountBalances {
    start: LayeredUsdAccountBalances,
    end: LayeredUsdAccountBalances,
    diff: LayeredUsdAccountBalances,
}

impl From<crate::ledger::account::RangedUsdAccountBalances> for RangedUsdAccountBalances {
    fn from(balances: crate::ledger::account::RangedUsdAccountBalances) -> Self {
        RangedUsdAccountBalances {
            start: balances.start.into(),
            end: balances.end.into(),
            diff: balances.diff.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountBalancesByCurrency {
    btc: RangedBtcAccountBalances,
    usd: RangedUsdAccountBalances,
}

impl From<crate::ledger::account::LedgerAccountBalancesByCurrency> for AccountBalancesByCurrency {
    fn from(balances: crate::ledger::account::LedgerAccountBalancesByCurrency) -> Self {
        AccountBalancesByCurrency {
            btc: balances.btc.into(),
            usd: balances.usd.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountWithBalance {
    pub id: UUID,
    pub name: String,
    pub balance: AccountBalancesByCurrency,
}

impl From<crate::ledger::account::LedgerAccountWithBalance> for AccountWithBalance {
    fn from(account_balance: crate::ledger::account::LedgerAccountWithBalance) -> Self {
        AccountWithBalance {
            id: account_balance.id.into(),
            name: account_balance.name,
            balance: account_balance.balance.into(),
        }
    }
}
