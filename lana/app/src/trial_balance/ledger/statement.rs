use cala_ledger::balance::AccountBalance;

use crate::primitives::{LedgerAccountSetId, Satoshis, UsdCents};

use super::TrialBalanceLedgerError;

#[derive(Clone)]
pub struct StatementAccountSet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalance,
    pub usd_balance: UsdStatementAccountSetBalance,
}

#[derive(Clone)]
pub struct StatementAccountSetWithAccounts {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalance,
    pub usd_balance: UsdStatementAccountSetBalance,
    pub accounts: Vec<StatementAccountSet>,
}

#[derive(Clone)]
pub struct BtcStatementAccountSetBalance {
    pub all: BtcStatementBalanceAmount,
    pub settled: BtcStatementBalanceAmount,
    pub pending: BtcStatementBalanceAmount,
    pub encumbrance: BtcStatementBalanceAmount,
}

impl TryFrom<AccountBalance> for BtcStatementAccountSetBalance {
    type Error = TrialBalanceLedgerError;

    fn try_from(balance: AccountBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            all: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(
                    balance.available(cala_ledger::Layer::Encumbrance),
                )?,
                dr_balance: Satoshis::try_from_btc(
                    balance
                        .details
                        .available(cala_ledger::Layer::Encumbrance)
                        .dr_balance,
                )?,
                cr_balance: Satoshis::try_from_btc(
                    balance
                        .details
                        .available(cala_ledger::Layer::Encumbrance)
                        .cr_balance,
                )?,
            },
            settled: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(balance.settled())?,
                dr_balance: Satoshis::try_from_btc(balance.details.settled.dr_balance)?,
                cr_balance: Satoshis::try_from_btc(balance.details.settled.cr_balance)?,
            },
            pending: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(balance.pending())?,
                dr_balance: Satoshis::try_from_btc(balance.details.pending.dr_balance)?,
                cr_balance: Satoshis::try_from_btc(balance.details.pending.cr_balance)?,
            },
            encumbrance: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(balance.encumbrance())?,
                dr_balance: Satoshis::try_from_btc(balance.details.encumbrance.dr_balance)?,
                cr_balance: Satoshis::try_from_btc(balance.details.encumbrance.cr_balance)?,
            },
        })
    }
}

impl BtcStatementAccountSetBalance {
    pub const ZERO: Self = Self {
        all: BtcStatementBalanceAmount::ZERO,
        settled: BtcStatementBalanceAmount::ZERO,
        pending: BtcStatementBalanceAmount::ZERO,
        encumbrance: BtcStatementBalanceAmount::ZERO,
    };
}

#[derive(Clone)]
pub struct UsdStatementAccountSetBalance {
    pub all: UsdStatementBalanceAmount,
    pub settled: UsdStatementBalanceAmount,
    pub pending: UsdStatementBalanceAmount,
    pub encumbrance: UsdStatementBalanceAmount,
}

impl TryFrom<AccountBalance> for UsdStatementAccountSetBalance {
    type Error = TrialBalanceLedgerError;

    fn try_from(balance: AccountBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            all: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(
                    balance.available(cala_ledger::Layer::Encumbrance),
                )?,
                dr_balance: UsdCents::try_from_usd(
                    balance
                        .details
                        .available(cala_ledger::Layer::Encumbrance)
                        .dr_balance,
                )?,
                cr_balance: UsdCents::try_from_usd(
                    balance
                        .details
                        .available(cala_ledger::Layer::Encumbrance)
                        .cr_balance,
                )?,
            },
            settled: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(balance.settled())?,
                dr_balance: UsdCents::try_from_usd(balance.details.settled.dr_balance)?,
                cr_balance: UsdCents::try_from_usd(balance.details.settled.cr_balance)?,
            },
            pending: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(balance.pending())?,
                dr_balance: UsdCents::try_from_usd(balance.details.pending.dr_balance)?,
                cr_balance: UsdCents::try_from_usd(balance.details.pending.cr_balance)?,
            },
            encumbrance: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(balance.encumbrance())?,
                dr_balance: UsdCents::try_from_usd(balance.details.encumbrance.dr_balance)?,
                cr_balance: UsdCents::try_from_usd(balance.details.encumbrance.cr_balance)?,
            },
        })
    }
}

impl UsdStatementAccountSetBalance {
    pub const ZERO: Self = Self {
        all: UsdStatementBalanceAmount::ZERO,
        settled: UsdStatementBalanceAmount::ZERO,
        pending: UsdStatementBalanceAmount::ZERO,
        encumbrance: UsdStatementBalanceAmount::ZERO,
    };
}

#[derive(Clone)]
pub struct BtcStatementBalanceAmount {
    pub normal_balance: Satoshis,
    pub dr_balance: Satoshis,
    pub cr_balance: Satoshis,
}

impl BtcStatementBalanceAmount {
    pub const ZERO: Self = Self {
        normal_balance: Satoshis::ZERO,
        dr_balance: Satoshis::ZERO,
        cr_balance: Satoshis::ZERO,
    };
}

#[derive(Clone)]
pub struct UsdStatementBalanceAmount {
    pub normal_balance: UsdCents,
    pub dr_balance: UsdCents,
    pub cr_balance: UsdCents,
}

impl UsdStatementBalanceAmount {
    pub const ZERO: Self = Self {
        normal_balance: UsdCents::ZERO,
        dr_balance: UsdCents::ZERO,
        cr_balance: UsdCents::ZERO,
    };
}
