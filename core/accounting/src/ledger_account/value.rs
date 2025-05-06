use std::collections::HashMap;

use crate::{
    AccountCode, BalanceRange, CalaAccount, CalaAccountBalance, CalaAccountId, CalaAccountSet,
    CalaAccountSetId, CalaBalanceId, CalaBalanceRange, CalaCurrency, CalaJournalId,
    LedgerAccountId,
};

#[derive(Debug, Clone)]
pub struct LedgerAccount {
    pub id: LedgerAccountId,
    pub name: String,
    pub code: Option<AccountCode>,
    pub btc_balance_range: Option<BalanceRange>,
    pub usd_balance_range: Option<BalanceRange>,

    pub ancestor_ids: Vec<LedgerAccountId>,
    pub children_ids: Vec<LedgerAccountId>,

    pub(super) cala_external_id: Option<String>,
    is_leaf: bool,
}

impl LedgerAccount {
    pub(super) fn account_set_member_id(&self) -> cala_ledger::account_set::AccountSetMemberId {
        if self.is_leaf {
            CalaAccountId::from(self.id).into()
        } else {
            CalaAccountSetId::from(self.id).into()
        }
    }

    pub(super) fn has_non_zero_activity(&self) -> bool {
        if let Some(usd) = self.usd_balance_range.as_ref() {
            usd.has_non_zero_activity()
        } else if let Some(btc) = self.btc_balance_range.as_ref() {
            btc.has_non_zero_activity()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ByCurrency<B> {
    pub(super) usd: Option<B>,
    pub(super) btc: Option<B>,
}

impl<B, I> From<(&HashMap<CalaBalanceId, B>, CalaJournalId, I)> for ByCurrency<B>
where
    B: Clone,
    I: Into<CalaAccountId>,
{
    fn from(
        (balances, journal_id, account_id): (&HashMap<CalaBalanceId, B>, CalaJournalId, I),
    ) -> Self {
        let account_id = account_id.into();
        let usd_key = (journal_id, account_id, CalaCurrency::USD);
        let btc_key = (journal_id, account_id, CalaCurrency::BTC);

        ByCurrency {
            usd: balances.get(&usd_key).cloned(),
            btc: balances.get(&btc_key).cloned(),
        }
    }
}

pub(super) type AccountBalances = ByCurrency<CalaAccountBalance>;
pub(super) type BalanceRanges = ByCurrency<CalaBalanceRange>;

impl From<(CalaAccountSet, AccountBalances)> for LedgerAccount {
    fn from(
        (
            account_set,
            AccountBalances {
                btc: btc_balance,
                usd: usd_balance,
            },
        ): (CalaAccountSet, AccountBalances),
    ) -> Self {
        let values = account_set.into_values();
        let external_id = values.external_id.clone();
        let code = values.external_id.and_then(|id| id.parse().ok());

        let usd_balance_range = usd_balance.map(|balance| BalanceRange {
            start: None,
            end: Some(balance.clone()),
            diff: Some(balance),
        });

        let btc_balance_range = btc_balance.map(|balance| BalanceRange {
            start: None,
            end: Some(balance.clone()),
            diff: Some(balance),
        });

        LedgerAccount {
            id: values.id.into(),
            name: values.name,
            code,
            btc_balance_range,
            usd_balance_range,
            ancestor_ids: Vec::new(),
            children_ids: Vec::new(),
            is_leaf: false,
            cala_external_id: external_id,
        }
    }
}

impl From<(CalaAccountSet, BalanceRanges)> for LedgerAccount {
    fn from(
        (
            account_set,
            BalanceRanges {
                usd: usd_balance_range,
                btc: btc_balance_range,
            },
        ): (CalaAccountSet, BalanceRanges),
    ) -> Self {
        let values = account_set.into_values();
        let external_id = values.external_id.clone();
        let code = values.external_id.and_then(|id| id.parse().ok());

        let usd_balance_range = usd_balance_range.map(|range| BalanceRange {
            start: Some(range.start),
            end: Some(range.end),
            diff: Some(range.diff),
        });
        let btc_balance_range = btc_balance_range.map(|range| BalanceRange {
            start: Some(range.start),
            end: Some(range.end),
            diff: Some(range.diff),
        });

        LedgerAccount {
            id: values.id.into(),
            name: values.name,
            code,
            btc_balance_range,
            usd_balance_range,
            ancestor_ids: Vec::new(),
            children_ids: Vec::new(),
            is_leaf: false,
            cala_external_id: external_id,
        }
    }
}

impl From<(CalaAccount, AccountBalances)> for LedgerAccount {
    fn from(
        (
            account,
            AccountBalances {
                usd: usd_balance,
                btc: btc_balance,
            },
        ): (CalaAccount, AccountBalances),
    ) -> Self {
        let usd_balance_range = usd_balance.map(|balance| BalanceRange {
            start: None,
            end: Some(balance.clone()),
            diff: Some(balance),
        });

        let btc_balance_range = btc_balance.map(|balance| BalanceRange {
            start: None,
            end: Some(balance.clone()),
            diff: Some(balance),
        });

        let external_id = account.values().external_id.clone();

        LedgerAccount {
            id: account.id.into(),
            name: account.into_values().name,
            code: None,
            usd_balance_range,
            btc_balance_range,
            ancestor_ids: Vec::new(),
            children_ids: Vec::new(),
            is_leaf: true,
            cala_external_id: external_id,
        }
    }
}

impl From<(CalaAccount, BalanceRanges)> for LedgerAccount {
    fn from(
        (
            account,
            BalanceRanges {
                usd: usd_balance_range,
                btc: btc_balance_range,
            },
        ): (CalaAccount, BalanceRanges),
    ) -> Self {
        let usd_balance_range = usd_balance_range.map(|range| BalanceRange {
            start: Some(range.start),
            end: Some(range.end),
            diff: Some(range.diff),
        });
        let btc_balance_range = btc_balance_range.map(|range| BalanceRange {
            start: Some(range.start),
            end: Some(range.end),
            diff: Some(range.diff),
        });

        let external_id = account.values().external_id.clone();

        LedgerAccount {
            id: account.id.into(),
            name: account.into_values().name,
            code: None,
            usd_balance_range,
            btc_balance_range,
            ancestor_ids: Vec::new(),
            children_ids: Vec::new(),
            is_leaf: true,
            cala_external_id: external_id,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod by_currency {
        use std::collections::HashMap;

        use super::*;

        #[derive(Debug, Clone)]
        struct DummyBalance(String);
        type DummyBalances = ByCurrency<DummyBalance>;

        #[test]
        fn get_both_usd_and_btc() {
            let journal_id = CalaJournalId::new();
            let account_id = CalaAccountId::new();
            let mut balances: HashMap<_, DummyBalance> = HashMap::new();

            balances.insert(
                (journal_id, account_id, CalaCurrency::USD),
                DummyBalance("USD".to_string()),
            );
            balances.insert(
                (journal_id, account_id, CalaCurrency::BTC),
                DummyBalance("BTC".to_string()),
            );

            let account_balances = DummyBalances::from((&balances, journal_id, account_id.clone()));

            assert_eq!(account_balances.usd.unwrap().0, "USD".to_string());
            assert_eq!(account_balances.btc.unwrap().0, "BTC".to_string());
        }

        #[test]
        fn get_only_usd() {
            let journal_id = CalaJournalId::new();
            let account_id = CalaAccountId::new();
            let mut balances: HashMap<_, DummyBalance> = HashMap::new();

            balances.insert(
                (journal_id, account_id, CalaCurrency::USD),
                DummyBalance("USD".to_string()),
            );

            let account_balances = DummyBalances::from((&balances, journal_id, account_id.clone()));

            assert_eq!(account_balances.usd.unwrap().0, "USD".to_string());
            assert!(account_balances.btc.is_none());
        }

        #[test]
        fn get_only_btc() {
            let journal_id = CalaJournalId::new();
            let account_id = CalaAccountId::new();
            let mut balances: HashMap<_, DummyBalance> = HashMap::new();

            balances.insert(
                (journal_id, account_id, CalaCurrency::BTC),
                DummyBalance("BTC".to_string()),
            );

            let account_balances = DummyBalances::from((&balances, journal_id, account_id.clone()));

            assert!(account_balances.usd.is_none());
            assert_eq!(account_balances.btc.unwrap().0, "BTC".to_string());
        }

        #[test]
        fn get_none_when_missing() {
            let journal_id = CalaJournalId::new();
            let account_id = CalaAccountId::new();
            let balances: HashMap<_, DummyBalance> = HashMap::new();

            let account_balances = DummyBalances::from((&balances, journal_id, account_id.clone()));

            assert!(account_balances.usd.is_none());
            assert!(account_balances.btc.is_none());
        }
    }
}
