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

impl<B> ByCurrency<B> {
    pub(super) fn extract_from_balances(
        balances: &mut HashMap<CalaBalanceId, B>,
        journal_id: CalaJournalId,
        account_id: impl Into<CalaAccountId>,
    ) -> Self {
        let account_id = account_id.into();
        let usd_key = (journal_id, account_id, CalaCurrency::USD);
        let btc_key = (journal_id, account_id, CalaCurrency::BTC);

        ByCurrency {
            usd: balances.remove(&usd_key),
            btc: balances.remove(&btc_key),
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
            open: None,
            close: Some(balance.clone()),
            period_activity: Some(balance),
        });

        let btc_balance_range = btc_balance.map(|balance| BalanceRange {
            open: None,
            close: Some(balance.clone()),
            period_activity: Some(balance),
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
            open: Some(range.open),
            close: Some(range.close),
            period_activity: Some(range.period),
        });
        let btc_balance_range = btc_balance_range.map(|range| BalanceRange {
            open: Some(range.open),
            close: Some(range.close),
            period_activity: Some(range.period),
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
            open: None,
            close: Some(balance.clone()),
            period_activity: Some(balance),
        });

        let btc_balance_range = btc_balance.map(|balance| BalanceRange {
            open: None,
            close: Some(balance.clone()),
            period_activity: Some(balance),
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
            open: Some(range.open),
            close: Some(range.close),
            period_activity: Some(range.period),
        });
        let btc_balance_range = btc_balance_range.map(|range| BalanceRange {
            open: Some(range.open),
            close: Some(range.close),
            period_activity: Some(range.period),
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

        struct DummyBalance(String);
        type DummyBalances = ByCurrency<DummyBalance>;

        #[test]
        fn extract_both_usd_and_btc() {
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

            let account_balances =
                DummyBalances::extract_from_balances(&mut balances, journal_id, account_id.clone());

            assert_eq!(account_balances.usd.unwrap().0, "USD".to_string());
            assert_eq!(account_balances.btc.unwrap().0, "BTC".to_string());

            assert!(balances.is_empty());
        }

        #[test]
        fn extract_only_usd() {
            let journal_id = CalaJournalId::new();
            let account_id = CalaAccountId::new();
            let mut balances: HashMap<_, DummyBalance> = HashMap::new();

            balances.insert(
                (journal_id, account_id, CalaCurrency::USD),
                DummyBalance("USD".to_string()),
            );

            let account_balances =
                DummyBalances::extract_from_balances(&mut balances, journal_id, account_id.clone());

            assert_eq!(account_balances.usd.unwrap().0, "USD".to_string());
            assert!(account_balances.btc.is_none());

            assert!(balances.is_empty());
        }

        #[test]
        fn extract_only_btc() {
            let journal_id = CalaJournalId::new();
            let account_id = CalaAccountId::new();
            let mut balances: HashMap<_, DummyBalance> = HashMap::new();

            balances.insert(
                (journal_id, account_id, CalaCurrency::BTC),
                DummyBalance("BTC".to_string()),
            );

            let account_balances =
                DummyBalances::extract_from_balances(&mut balances, journal_id, account_id.clone());

            assert!(account_balances.usd.is_none());
            assert_eq!(account_balances.btc.unwrap().0, "BTC".to_string());

            assert!(balances.is_empty());
        }

        #[test]
        fn extract_none_when_missing() {
            let journal_id = CalaJournalId::new();
            let account_id = CalaAccountId::new();
            let mut balances: HashMap<_, DummyBalance> = HashMap::new();

            let account_balances =
                DummyBalances::extract_from_balances(&mut balances, journal_id, account_id.clone());

            assert!(account_balances.usd.is_none());
            assert!(account_balances.btc.is_none());
            assert!(balances.is_empty());
        }
    }
}
