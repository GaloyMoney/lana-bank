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

impl
    From<(
        CalaAccountSet,
        Option<CalaAccountBalance>,
        Option<CalaAccountBalance>,
    )> for LedgerAccount
{
    fn from(
        (account_set, usd_balance, btc_balance): (
            CalaAccountSet,
            Option<CalaAccountBalance>,
            Option<CalaAccountBalance>,
        ),
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

impl
    From<(
        CalaAccountSet,
        Option<CalaBalanceRange>,
        Option<CalaBalanceRange>,
    )> for LedgerAccount
{
    fn from(
        (account_set, usd_balance_range, btc_balance_range): (
            CalaAccountSet,
            Option<CalaBalanceRange>,
            Option<CalaBalanceRange>,
        ),
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

impl
    From<(
        CalaAccount,
        Option<CalaAccountBalance>,
        Option<CalaAccountBalance>,
    )> for LedgerAccount
{
    fn from(
        (account, usd_balance, btc_balance): (
            CalaAccount,
            Option<CalaAccountBalance>,
            Option<CalaAccountBalance>,
        ),
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

impl
    From<(
        CalaAccount,
        Option<CalaBalanceRange>,
        Option<CalaBalanceRange>,
    )> for LedgerAccount
{
    fn from(
        (account, usd_balance_range, btc_balance_range): (
            CalaAccount,
            Option<CalaBalanceRange>,
            Option<CalaBalanceRange>,
        ),
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
