use async_graphql::*;

use crate::primitives::*;

#[derive(SimpleObject)]
pub struct Account {
    pub id: UUID,
    pub name: String,
    pub amounts: AccountAmountsByCurrency,
}

// impl From<lana_app::ledger::account::LedgerAccountWithBalance> for Account {
//     fn from(account_balance: lana_app::ledger::account::LedgerAccountWithBalance) -> Self {
//         Account {
//             id: account_balance.id.into(),
//             name: account_balance.name,
//             amounts: account_balance.balance.into(),
//         }
//     }
// }

#[derive(SimpleObject)]
struct BtcAccountAmounts {
    debit: Satoshis,
    credit: Satoshis,
    net_debit: SignedSatoshis,
    net_credit: SignedSatoshis,
}

// impl From<lana_app::ledger::account::BtcAccountBalance> for BtcAccountAmounts {
//     fn from(balance: lana_app::ledger::account::BtcAccountBalance) -> Self {
//         BtcAccountAmounts {
//             debit: balance.debit,
//             credit: balance.credit,
//             net_debit: balance.net_debit,
//             net_credit: balance.net_credit,
//         }
//     }
// }

#[derive(SimpleObject)]
struct UsdAccountAmounts {
    debit: UsdCents,
    credit: UsdCents,
    net_debit: SignedUsdCents,
    net_credit: SignedUsdCents,
}

// impl From<lana_app::ledger::account::UsdAccountBalance> for UsdAccountAmounts {
//     fn from(balance: lana_app::ledger::account::UsdAccountBalance) -> Self {
//         UsdAccountAmounts {
//             debit: balance.debit,
//             credit: balance.credit,
//             net_debit: balance.net_debit,
//             net_credit: balance.net_credit,
//         }
//     }
// }

#[derive(SimpleObject)]
struct LayeredBtcAccountAmounts {
    all: BtcAccountAmounts,
    settled: BtcAccountAmounts,
    pending: BtcAccountAmounts,
    encumbrance: BtcAccountAmounts,
}

// impl From<lana_app::ledger::account::LayeredBtcAccountBalances> for LayeredBtcAccountAmounts {
//     fn from(balances: lana_app::ledger::account::LayeredBtcAccountBalances) -> Self {
//         LayeredBtcAccountAmounts {
//             all: balances.all_layers.into(),
//             settled: balances.settled.into(),
//             pending: balances.pending.into(),
//             encumbrance: balances.encumbrance.into(),
//         }
//     }
// }

#[derive(SimpleObject)]
struct LayeredUsdAccountAmounts {
    all: UsdAccountAmounts,
    settled: UsdAccountAmounts,
    pending: UsdAccountAmounts,
    encumbrance: UsdAccountAmounts,
}

// impl From<lana_app::ledger::account::LayeredUsdAccountBalances> for LayeredUsdAccountAmounts {
//     fn from(balances: lana_app::ledger::account::LayeredUsdAccountBalances) -> Self {
//         LayeredUsdAccountAmounts {
//             all: balances.all_layers.into(),
//             settled: balances.settled.into(),
//             pending: balances.pending.into(),
//             encumbrance: balances.encumbrance.into(),
//         }
//     }
// }

#[derive(SimpleObject)]
pub struct BtcAccountAmountsInPeriod {
    opening_balance: LayeredBtcAccountAmounts,
    closing_balance: LayeredBtcAccountAmounts,
    amount: LayeredBtcAccountAmounts,
}

// impl From<lana_app::ledger::account::RangedBtcAccountBalances> for BtcAccountAmountsInPeriod {
//     fn from(balances: lana_app::ledger::account::RangedBtcAccountBalances) -> Self {
//         BtcAccountAmountsInPeriod {
//             opening_balance: balances.start.into(),
//             closing_balance: balances.end.into(),
//             amount: balances.diff.into(),
//         }
//     }
// }

#[derive(SimpleObject)]
pub struct UsdAccountAmountsInPeriod {
    opening_balance: LayeredUsdAccountAmounts,
    closing_balance: LayeredUsdAccountAmounts,
    amount: LayeredUsdAccountAmounts,
}

// impl From<lana_app::ledger::account::RangedUsdAccountBalances> for UsdAccountAmountsInPeriod {
//     fn from(balances: lana_app::ledger::account::RangedUsdAccountBalances) -> Self {
//         UsdAccountAmountsInPeriod {
//             opening_balance: balances.start.into(),
//             closing_balance: balances.end.into(),
//             amount: balances.diff.into(),
//         }
//     }
// }

#[derive(SimpleObject)]
pub struct AccountAmountsByCurrency {
    btc: BtcAccountAmountsInPeriod,
    usd: UsdAccountAmountsInPeriod,
}

// impl From<lana_app::ledger::account::LedgerAccountBalancesByCurrency> for AccountAmountsByCurrency {
//     fn from(balances: lana_app::ledger::account::LedgerAccountBalancesByCurrency) -> Self {
//         AccountAmountsByCurrency {
//             btc: balances.btc.into(),
//             usd: balances.usd.into(),
//         }
//     }
// }
