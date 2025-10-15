use async_graphql::*;

use crate::{graphql::loader::CHART_REF, primitives::*};

use lana_app::trial_balance::TrialBalanceRow as DomainTrialBalanceRow;

use super::{
    AccountCode, BtcLedgerAccountBalanceRange, LedgerAccount, LedgerAccountBalanceRange,
    LedgerAccountBalanceRangeByCurrency, UsdLedgerAccountBalanceRange,
};

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct TrialBalanceRow {
    id: ID,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainTrialBalanceRow>,
}

impl From<DomainTrialBalanceRow> for TrialBalanceRow {
    fn from(row: DomainTrialBalanceRow) -> Self {
        TrialBalanceRow {
            id: row.id.to_global_id(),
            ledger_account_id: UUID::from(row.id),
            code: row.code.as_ref().map(|code| code.into()),
            name: row.name.clone(),
            entity: Arc::new(row),
        }
    }
}

#[ComplexObject]
impl TrialBalanceRow {
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

    // TODO: rename to just "accounts or something else" after removing the old accounts field above
    pub async fn accounts_flat(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<TrialBalanceRow>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let accounts = app
            .accounting()
            .trial_balance_accounts_flat(
                sub,
                CHART_REF.0,
                self.from.into_inner(),
                Some(self.until.into_inner()),
            )
            .await?;
        Ok(accounts.into_iter().map(TrialBalanceRow::from).collect())
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
