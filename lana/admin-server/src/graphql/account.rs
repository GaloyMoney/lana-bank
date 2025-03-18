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
struct BtcAccountBalanceAmounts {
    debit: Satoshis,
    credit: Satoshis,
    net_debit: SignedSatoshis,
    net_credit: SignedSatoshis,
}

impl From<lana_app::statement::BtcStatementBalanceAmount> for BtcAccountBalanceAmounts {
    fn from(balance: lana_app::statement::BtcStatementBalanceAmount) -> Self {
        BtcAccountBalanceAmounts {
            debit: balance.dr_balance,
            credit: balance.cr_balance,
            net_debit: balance.net_dr_balance,
            net_credit: balance.net_cr_balance,
        }
    }
}

#[derive(SimpleObject)]
struct UsdAccountBalanceAmounts {
    debit: UsdCents,
    credit: UsdCents,
    net_debit: SignedUsdCents,
    net_credit: SignedUsdCents,
}

impl From<lana_app::statement::UsdStatementBalanceAmount> for UsdAccountBalanceAmounts {
    fn from(balance: lana_app::statement::UsdStatementBalanceAmount) -> Self {
        UsdAccountBalanceAmounts {
            debit: balance.dr_balance,
            credit: balance.cr_balance,
            net_debit: balance.net_dr_balance,
            net_credit: balance.net_cr_balance,
        }
    }
}

#[derive(SimpleObject)]
struct LayeredBtcAccountBalanceAmounts {
    all: BtcAccountBalanceAmounts,
    settled: BtcAccountBalanceAmounts,
    pending: BtcAccountBalanceAmounts,
    encumbrance: BtcAccountBalanceAmounts,
}

impl From<lana_app::statement::BtcStatementAccountSetBalance> for LayeredBtcAccountBalanceAmounts {
    fn from(balances: lana_app::statement::BtcStatementAccountSetBalance) -> Self {
        LayeredBtcAccountBalanceAmounts {
            all: balances.all.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct LayeredUsdAccountBalanceAmounts {
    all: UsdAccountBalanceAmounts,
    settled: UsdAccountBalanceAmounts,
    pending: UsdAccountBalanceAmounts,
    encumbrance: UsdAccountBalanceAmounts,
}

impl From<lana_app::statement::UsdStatementAccountSetBalance> for LayeredUsdAccountBalanceAmounts {
    fn from(balances: lana_app::statement::UsdStatementAccountSetBalance) -> Self {
        LayeredUsdAccountBalanceAmounts {
            all: balances.all.into(),
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct BtcAccountAmountsInPeriod {
    opening_balance: LayeredBtcAccountBalanceAmounts,
    closing_balance: LayeredBtcAccountBalanceAmounts,
    amount: LayeredBtcAccountBalanceAmounts,
}

impl From<lana_app::statement::BtcStatementAccountSetBalanceRange> for BtcAccountAmountsInPeriod {
    fn from(balances: lana_app::statement::BtcStatementAccountSetBalanceRange) -> Self {
        BtcAccountAmountsInPeriod {
            opening_balance: balances.start.into(),
            closing_balance: balances.end.into(),
            amount: balances.diff.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct UsdAccountAmountsInPeriod {
    opening_balance: LayeredUsdAccountBalanceAmounts,
    closing_balance: LayeredUsdAccountBalanceAmounts,
    amount: LayeredUsdAccountBalanceAmounts,
}

impl From<lana_app::statement::UsdStatementAccountSetBalanceRange> for UsdAccountAmountsInPeriod {
    fn from(balances: lana_app::statement::UsdStatementAccountSetBalanceRange) -> Self {
        UsdAccountAmountsInPeriod {
            opening_balance: balances.start.into(),
            closing_balance: balances.end.into(),
            amount: balances.diff.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountAmountsByCurrency {
    btc: BtcAccountAmountsInPeriod,
    usd: UsdAccountAmountsInPeriod,
}

impl From<lana_app::trial_balance::TrialBalanceAccountSet> for AccountAmountsByCurrency {
    fn from(balances: lana_app::trial_balance::TrialBalanceAccountSet) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}

impl From<lana_app::statement::StatementAccountSet> for AccountAmountsByCurrency {
    fn from(balances: lana_app::statement::StatementAccountSet) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}

impl From<lana_app::statement::StatementAccountSetWithAccounts> for AccountAmountsByCurrency {
    fn from(balances: lana_app::statement::StatementAccountSetWithAccounts) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}

impl From<lana_app::trial_balance::TrialBalanceRoot> for AccountAmountsByCurrency {
    fn from(balances: lana_app::trial_balance::TrialBalanceRoot) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}

impl From<lana_app::profit_and_loss::ProfitAndLossStatement> for AccountAmountsByCurrency {
    fn from(balances: lana_app::profit_and_loss::ProfitAndLossStatement) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}

impl From<lana_app::balance_sheet::BalanceSheet> for AccountAmountsByCurrency {
    fn from(balances: lana_app::balance_sheet::BalanceSheet) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}

impl From<lana_app::cash_flow::CashFlowStatement> for AccountAmountsByCurrency {
    fn from(balances: lana_app::cash_flow::CashFlowStatement) -> Self {
        AccountAmountsByCurrency {
            btc: balances.btc_balance.into(),
            usd: balances.usd_balance.into(),
        }
    }
}
