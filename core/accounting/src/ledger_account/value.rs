use crate::{
    AccountCode, BalanceRange, CalaAccount, CalaAccountBalance, CalaAccountId, CalaAccountSet,
    CalaAccountSetId, CalaBalanceRange, LedgerAccountId,
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

pub(super) struct AccountBalances {
    pub(super) btc: Option<CalaAccountBalance>,
    pub(super) usd: Option<CalaAccountBalance>,
}

pub(super) struct BalanceRanges {
    pub(super) btc: Option<CalaBalanceRange>,
    pub(super) usd: Option<CalaBalanceRange>,
}

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
