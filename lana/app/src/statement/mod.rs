pub mod error;

use std::collections::HashMap;

use cala_ledger::{
    balance::{AccountBalance, BalanceRange},
    AccountId, BalanceId, Currency, JournalId,
};

use crate::primitives::{LedgerAccountSetId, Satoshis, SignedSatoshis, SignedUsdCents, UsdCents};

use error::*;

#[derive(Debug, Clone)]
pub struct StatementAccountSetDetails {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StatementAccountSet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
}

#[derive(Debug, Clone)]
pub struct StatementAccountSetWithAccounts {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub accounts: Vec<StatementAccountSet>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct BtcStatementAccountSetBalanceRange {
    pub start: BtcStatementAccountSetBalance,
    pub end: BtcStatementAccountSetBalance,
    pub diff: BtcStatementAccountSetBalance,
}

impl TryFrom<BalanceRange> for BtcStatementAccountSetBalanceRange {
    type Error = StatementError;

    fn try_from(range: BalanceRange) -> Result<Self, Self::Error> {
        Ok(Self {
            start: range.start.try_into()?,
            end: range.end.try_into()?,
            diff: range.diff.try_into()?,
        })
    }
}

impl BtcStatementAccountSetBalanceRange {
    pub const ZERO: Self = Self {
        start: BtcStatementAccountSetBalance::ZERO,
        end: BtcStatementAccountSetBalance::ZERO,
        diff: BtcStatementAccountSetBalance::ZERO,
    };
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct UsdStatementAccountSetBalanceRange {
    pub start: UsdStatementAccountSetBalance,
    pub end: UsdStatementAccountSetBalance,
    pub diff: UsdStatementAccountSetBalance,
}

impl TryFrom<BalanceRange> for UsdStatementAccountSetBalanceRange {
    type Error = StatementError;

    fn try_from(range: BalanceRange) -> Result<Self, Self::Error> {
        Ok(Self {
            start: range.start.try_into()?,
            end: range.end.try_into()?,
            diff: range.diff.try_into()?,
        })
    }
}

impl UsdStatementAccountSetBalanceRange {
    pub const ZERO: Self = Self {
        start: UsdStatementAccountSetBalance::ZERO,
        end: UsdStatementAccountSetBalance::ZERO,
        diff: UsdStatementAccountSetBalance::ZERO,
    };
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct BalancesByAccount {
    balances: HashMap<AccountId, HashMap<Currency, Option<BalanceRange>>>,
}

impl BalancesByAccount {
    fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    fn insert(&mut self, account_id: AccountId, currency: Currency, balance: Option<BalanceRange>) {
        self.balances
            .entry(account_id)
            .or_default()
            .insert(currency, balance);
    }

    pub fn btc_for_account(
        &self,
        account_set_id: LedgerAccountSetId,
    ) -> Result<BtcStatementAccountSetBalanceRange, StatementError> {
        let currency = "BTC".parse().expect("BTC is not a valid currency");
        Ok(self
            .balances
            .get(&account_set_id.into())
            .and_then(|currencies| currencies.get(&currency))
            .into_iter()
            .flatten()
            .map(|bal| bal.clone().try_into())
            .next()
            .transpose()?
            .unwrap_or(BtcStatementAccountSetBalanceRange::ZERO))
    }

    pub fn usd_for_account(
        &self,
        account_set_id: LedgerAccountSetId,
    ) -> Result<UsdStatementAccountSetBalanceRange, StatementError> {
        let currency = "USD".parse().expect("USD is not a valid currency");
        Ok(self
            .balances
            .get(&account_set_id.into())
            .and_then(|currencies| currencies.get(&currency))
            .into_iter()
            .flatten()
            .map(|bal| bal.clone().try_into())
            .next()
            .transpose()?
            .unwrap_or(UsdStatementAccountSetBalanceRange::ZERO))
    }
}

impl From<HashMap<BalanceId, BalanceRange>> for BalancesByAccount {
    fn from(all_balances: HashMap<BalanceId, BalanceRange>) -> Self {
        let mut balances_by_account = Self::new();
        for ((_, account_id, currency), balance) in all_balances {
            balances_by_account.insert(account_id, currency, Some(balance));
        }

        balances_by_account
    }
}

impl From<HashMap<BalanceId, Option<BalanceRange>>> for BalancesByAccount {
    fn from(all_balances: HashMap<BalanceId, Option<BalanceRange>>) -> Self {
        let mut balances_by_account = Self::new();
        for ((_, account_id, currency), balance) in all_balances {
            balances_by_account.insert(account_id, currency, balance);
        }

        balances_by_account
    }
}

struct BalanceIdsForSingleAccountSet {
    balance_ids: Vec<BalanceId>,
}

impl From<(JournalId, LedgerAccountSetId)> for BalanceIdsForSingleAccountSet {
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

pub struct BalanceIdsForAccountSets {
    pub balance_ids: Vec<BalanceId>,
}

impl From<(JournalId, Vec<LedgerAccountSetId>)> for BalanceIdsForAccountSets {
    fn from(values: (JournalId, Vec<LedgerAccountSetId>)) -> Self {
        let journal_id = values.0;
        let account_set_ids = values.1;
        Self {
            balance_ids: account_set_ids
                .into_iter()
                .flat_map(|account_set_id| {
                    BalanceIdsForSingleAccountSet::from((journal_id, account_set_id)).balance_ids
                })
                .collect(),
        }
    }
}
