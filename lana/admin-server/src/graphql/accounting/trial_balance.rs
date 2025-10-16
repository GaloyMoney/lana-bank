use async_graphql::*;

use crate::{graphql::loader::CHART_REF, primitives::*};

use lana_app::trial_balance::TrialBalanceEntry as DomainTrialBalanceEntry;

use super::{
    AccountCode, BtcLedgerAccountBalanceRange, LedgerAccount, LedgerAccountBalanceRange,
    LedgerAccountBalanceRangeByCurrency, UsdLedgerAccountBalanceRange,
};

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct TrialBalanceEntry {
    id: ID,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainTrialBalanceEntry>,
}

impl From<DomainTrialBalanceEntry> for TrialBalanceEntry {
    fn from(row: DomainTrialBalanceEntry) -> Self {
        TrialBalanceEntry {
            id: row.id.to_global_id(),
            ledger_account_id: UUID::from(row.id),
            code: row.code.as_ref().map(|code| code.into()),
            name: row.name.clone(),
            entity: Arc::new(row),
        }
    }
}

#[ComplexObject]
impl TrialBalanceEntry {
    async fn balance_range(&self) -> LedgerAccountBalanceRange {
        if self.entity.btc_balance_range.is_some() {
            self.entity.btc_balance_range.as_ref().into()
        } else {
            self.entity.usd_balance_range.as_ref().into()
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct TrialBalance {
    name: String,

    #[graphql(skip)]
    from: Date,
    #[graphql(skip)]
    until: Date,
    #[graphql(skip)]
    entity: Arc<lana_app::trial_balance::TrialBalanceRoot>,
}

#[ComplexObject]
impl TrialBalance {
    async fn total(&self) -> async_graphql::Result<LedgerAccountBalanceRangeByCurrency> {
        Ok(LedgerAccountBalanceRangeByCurrency {
            usd: self
                .entity
                .usd_balance_range
                .as_ref()
                .map(UsdLedgerAccountBalanceRange::from)
                .unwrap_or_default(),
            btc: self
                .entity
                .btc_balance_range
                .as_ref()
                .map(BtcLedgerAccountBalanceRange::from)
                .unwrap_or_default(),
        })
    }

    pub async fn accounts(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<LedgerAccount>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let accounts = app
            .accounting()
            .list_all_account_children(
                sub,
                CHART_REF.0,
                self.entity.id,
                self.from.into_inner(),
                Some(self.until.into_inner()),
            )
            .await?;
        Ok(accounts.into_iter().map(LedgerAccount::from).collect())
    }

    pub async fn entries(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<TrialBalanceEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let accounts = app
            .accounting()
            .list_trial_balance_entries(
                sub,
                CHART_REF.0,
                self.from.into_inner(),
                Some(self.until.into_inner()),
            )
            .await?;
        Ok(accounts.into_iter().map(TrialBalanceEntry::from).collect())
    }
}

impl From<lana_app::trial_balance::TrialBalanceRoot> for TrialBalance {
    fn from(trial_balance: lana_app::trial_balance::TrialBalanceRoot) -> Self {
        TrialBalance {
            name: trial_balance.name.to_string(),
            from: trial_balance.from.into(),
            until: trial_balance
                .until
                .expect("Mandatory 'until' value missing")
                .into(),
            entity: Arc::new(trial_balance),
        }
    }
}
