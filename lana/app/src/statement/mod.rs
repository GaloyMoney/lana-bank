pub mod error;

use std::collections::HashMap;

use cala_ledger::{balance::AccountBalance, AccountId, BalanceId, Currency, JournalId};

use crate::primitives::{LedgerAccountSetId, Satoshis, SignedSatoshis, SignedUsdCents, UsdCents};

use error::*;

#[derive(Clone)]
pub struct StatementAccountSetDetails {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
}

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
    type Error = StatementError;

    fn try_from(balance: AccountBalance) -> Result<Self, Self::Error> {
        let all_details = balance.details.available(cala_ledger::Layer::Encumbrance);

        Ok(Self {
            all: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(
                    balance.available(cala_ledger::Layer::Encumbrance),
                )?,
                dr_balance: Satoshis::try_from_btc(all_details.dr_balance)?,
                cr_balance: Satoshis::try_from_btc(all_details.cr_balance)?,
                net_dr_balance: SignedSatoshis::from_btc(
                    all_details.dr_balance - all_details.cr_balance,
                ),
                net_cr_balance: SignedSatoshis::from_btc(
                    all_details.cr_balance - all_details.dr_balance,
                ),
            },
            settled: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(balance.settled())?,
                dr_balance: Satoshis::try_from_btc(balance.details.settled.dr_balance)?,
                cr_balance: Satoshis::try_from_btc(balance.details.settled.cr_balance)?,
                net_dr_balance: SignedSatoshis::from_btc(
                    balance.details.settled.dr_balance - balance.details.settled.cr_balance,
                ),
                net_cr_balance: SignedSatoshis::from_btc(
                    balance.details.settled.cr_balance - balance.details.settled.dr_balance,
                ),
            },
            pending: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(balance.pending())?,
                dr_balance: Satoshis::try_from_btc(balance.details.pending.dr_balance)?,
                cr_balance: Satoshis::try_from_btc(balance.details.pending.cr_balance)?,
                net_dr_balance: SignedSatoshis::from_btc(
                    balance.details.pending.dr_balance - balance.details.pending.cr_balance,
                ),
                net_cr_balance: SignedSatoshis::from_btc(
                    balance.details.pending.cr_balance - balance.details.pending.dr_balance,
                ),
            },
            encumbrance: BtcStatementBalanceAmount {
                normal_balance: Satoshis::try_from_btc(balance.encumbrance())?,
                dr_balance: Satoshis::try_from_btc(balance.details.encumbrance.dr_balance)?,
                cr_balance: Satoshis::try_from_btc(balance.details.encumbrance.cr_balance)?,
                net_dr_balance: SignedSatoshis::from_btc(
                    balance.details.encumbrance.dr_balance - balance.details.encumbrance.cr_balance,
                ),
                net_cr_balance: SignedSatoshis::from_btc(
                    balance.details.encumbrance.cr_balance - balance.details.encumbrance.dr_balance,
                ),
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
    type Error = StatementError;

    fn try_from(balance: AccountBalance) -> Result<Self, Self::Error> {
        let all_details = balance.details.available(cala_ledger::Layer::Encumbrance);

        Ok(Self {
            all: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(
                    balance.available(cala_ledger::Layer::Encumbrance),
                )?,
                dr_balance: UsdCents::try_from_usd(all_details.dr_balance)?,
                cr_balance: UsdCents::try_from_usd(all_details.cr_balance)?,
                net_dr_balance: SignedUsdCents::from_usd(
                    all_details.dr_balance - all_details.cr_balance,
                ),
                net_cr_balance: SignedUsdCents::from_usd(
                    all_details.cr_balance - all_details.dr_balance,
                ),
            },
            settled: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(balance.settled())?,
                dr_balance: UsdCents::try_from_usd(balance.details.settled.dr_balance)?,
                cr_balance: UsdCents::try_from_usd(balance.details.settled.cr_balance)?,
                net_dr_balance: SignedUsdCents::from_usd(
                    balance.details.settled.dr_balance - balance.details.settled.cr_balance,
                ),
                net_cr_balance: SignedUsdCents::from_usd(
                    balance.details.settled.cr_balance - balance.details.settled.dr_balance,
                ),
            },
            pending: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(balance.pending())?,
                dr_balance: UsdCents::try_from_usd(balance.details.pending.dr_balance)?,
                cr_balance: UsdCents::try_from_usd(balance.details.pending.cr_balance)?,
                net_dr_balance: SignedUsdCents::from_usd(
                    balance.details.pending.dr_balance - balance.details.pending.cr_balance,
                ),
                net_cr_balance: SignedUsdCents::from_usd(
                    balance.details.pending.cr_balance - balance.details.pending.dr_balance,
                ),
            },
            encumbrance: UsdStatementBalanceAmount {
                normal_balance: UsdCents::try_from_usd(balance.encumbrance())?,
                dr_balance: UsdCents::try_from_usd(balance.details.encumbrance.dr_balance)?,
                cr_balance: UsdCents::try_from_usd(balance.details.encumbrance.cr_balance)?,
                net_dr_balance: SignedUsdCents::from_usd(
                    balance.details.encumbrance.dr_balance - balance.details.encumbrance.cr_balance,
                ),
                net_cr_balance: SignedUsdCents::from_usd(
                    balance.details.encumbrance.cr_balance - balance.details.encumbrance.dr_balance,
                ),
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
    pub net_dr_balance: SignedSatoshis,
    pub net_cr_balance: SignedSatoshis,
}

impl BtcStatementBalanceAmount {
    pub const ZERO: Self = Self {
        normal_balance: Satoshis::ZERO,
        dr_balance: Satoshis::ZERO,
        cr_balance: Satoshis::ZERO,
        net_dr_balance: SignedSatoshis::ZERO,
        net_cr_balance: SignedSatoshis::ZERO,
    };
}

#[derive(Clone)]
pub struct UsdStatementBalanceAmount {
    pub normal_balance: UsdCents,
    pub dr_balance: UsdCents,
    pub cr_balance: UsdCents,
    pub net_dr_balance: SignedUsdCents,
    pub net_cr_balance: SignedUsdCents,
}

impl UsdStatementBalanceAmount {
    pub const ZERO: Self = Self {
        normal_balance: UsdCents::ZERO,
        dr_balance: UsdCents::ZERO,
        cr_balance: UsdCents::ZERO,
        net_dr_balance: SignedUsdCents::ZERO,
        net_cr_balance: SignedUsdCents::ZERO,
    };
}

#[derive(Clone)]
pub struct BalancesByAccount {
    balances: HashMap<AccountId, HashMap<Currency, AccountBalance>>,
}

impl BalancesByAccount {
    fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    fn insert(&mut self, account_id: AccountId, currency: Currency, balance: AccountBalance) {
        self.balances
            .entry(account_id)
            .or_default()
            .insert(currency, balance);
    }

    pub fn btc_for_account(
        &self,
        account_set_id: LedgerAccountSetId,
    ) -> Result<BtcStatementAccountSetBalance, StatementError> {
        let currency = "BTC".parse().expect("BTC is not a valid currency");
        Ok(
            match self
                .balances
                .get(&account_set_id.into())
                .and_then(|currencies| currencies.get(&currency))
            {
                Some(bal) => bal.clone().try_into()?,
                None => BtcStatementAccountSetBalance::ZERO,
            },
        )
    }

    pub fn usd_for_account(
        &self,
        account_set_id: LedgerAccountSetId,
    ) -> Result<UsdStatementAccountSetBalance, StatementError> {
        let currency = "USD".parse().expect("USD is not a valid currency");
        Ok(
            match self
                .balances
                .get(&account_set_id.into())
                .and_then(|currencies| currencies.get(&currency))
            {
                Some(bal) => bal.clone().try_into()?,
                None => UsdStatementAccountSetBalance::ZERO,
            },
        )
    }
}

impl From<HashMap<BalanceId, AccountBalance>> for BalancesByAccount {
    fn from(all_balances: HashMap<BalanceId, AccountBalance>) -> Self {
        let mut balances_by_account = Self::new();
        for ((_, account_id, currency), balance) in all_balances {
            balances_by_account.insert(account_id, currency, balance);
        }

        balances_by_account
    }
}

pub struct BalanceIdsForAccountSet {
    pub balance_ids: Vec<BalanceId>,
}

impl From<(JournalId, LedgerAccountSetId)> for BalanceIdsForAccountSet {
    fn from(ids: (JournalId, LedgerAccountSetId)) -> Self {
        let journal_id = ids.0;
        let account_set_id = ids.1;
        Self {
            balance_ids: vec![
                (
                    journal_id,
                    account_set_id.into(),
                    "BTC".parse().expect("BTC is not a valid currency"),
                ),
                (
                    journal_id,
                    account_set_id.into(),
                    "USD".parse().expect("USD is not a valid currency"),
                ),
            ],
        }
    }
}
