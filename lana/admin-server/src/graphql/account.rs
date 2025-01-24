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

impl From<lana_app::trial_balance::BtcStatementBalanceAmount> for BtcAccountAmounts {
    fn from(balance: lana_app::trial_balance::BtcStatementBalanceAmount) -> Self {
        BtcAccountAmounts {
            debit: balance.dr_balance.into(),
            credit: balance.cr_balance.into(),
            net_debit: (balance.dr_balance - balance.cr_balance).into(),
            net_credit: (balance.cr_balance - balance.dr_balance).into(),
        }
    }
}

#[derive(SimpleObject)]
struct UsdAccountAmounts {
    debit: UsdCents,
    credit: UsdCents,
    net_debit: SignedUsdCents,
    net_credit: SignedUsdCents,
}

impl From<lana_app::trial_balance::UsdStatementBalanceAmount> for UsdAccountAmounts {
    fn from(balance: lana_app::trial_balance::UsdStatementBalanceAmount) -> Self {
        UsdAccountAmounts {
            debit: balance.dr_balance.into(),
            credit: balance.cr_balance.into(),
            net_debit: (balance.dr_balance - balance.cr_balance).into(),
            net_credit: (balance.cr_balance - balance.dr_balance).into(),
        }
    }
}

#[derive(SimpleObject)]
struct LayeredBtcAccountAmounts {
    all: BtcAccountAmounts,
    settled: BtcAccountAmounts,
    pending: BtcAccountAmounts,
    encumbrance: BtcAccountAmounts,
}

impl From<lana_app::trial_balance::BtcStatementAccountSetBalance> for LayeredBtcAccountAmounts {
    fn from(balances: lana_app::trial_balance::BtcStatementAccountSetBalance) -> Self {
        LayeredBtcAccountAmounts {
            all: balances.all.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct LayeredUsdAccountAmounts {
    all: UsdAccountAmounts,
    settled: UsdAccountAmounts,
    pending: UsdAccountAmounts,
    encumbrance: UsdAccountAmounts,
}

impl From<lana_app::trial_balance::UsdStatementAccountSetBalance> for LayeredUsdAccountAmounts {
    fn from(balances: lana_app::trial_balance::UsdStatementAccountSetBalance) -> Self {
        LayeredUsdAccountAmounts {
            all: balances.all.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct BtcAccountAmountsInPeriod {
    opening_balance: LayeredBtcAccountAmounts,
    closing_balance: LayeredBtcAccountAmounts,
    amount: LayeredBtcAccountAmounts,
}

// FIXME: Adjust for ranged balance from domain
impl From<lana_app::trial_balance::BtcStatementAccountSetBalance> for BtcAccountAmountsInPeriod {
    fn from(balances: lana_app::trial_balance::BtcStatementAccountSetBalance) -> Self {
        BtcAccountAmountsInPeriod {
            opening_balance: balances.clone().into(),
            closing_balance: balances.clone().into(),
            amount: balances.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct UsdAccountAmountsInPeriod {
    opening_balance: LayeredUsdAccountAmounts,
    closing_balance: LayeredUsdAccountAmounts,
    amount: LayeredUsdAccountAmounts,
}

// FIXME: Adjust for ranged balance from domain
impl From<lana_app::trial_balance::UsdStatementAccountSetBalance> for UsdAccountAmountsInPeriod {
    fn from(balances: lana_app::trial_balance::UsdStatementAccountSetBalance) -> Self {
        UsdAccountAmountsInPeriod {
            opening_balance: balances.clone().into(),
            closing_balance: balances.clone().into(),
            amount: balances.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountAmountsByCurrency {
    btc: BtcAccountAmountsInPeriod,
    usd: UsdAccountAmountsInPeriod,
}

impl From<lana_app::trial_balance::TrialBalance> for AccountAmountsByCurrency {
    fn from(balances: lana_app::trial_balance::TrialBalance) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}

impl From<lana_app::trial_balance::StatementAccountSet> for AccountAmountsByCurrency {
    fn from(balances: lana_app::trial_balance::StatementAccountSet) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}
