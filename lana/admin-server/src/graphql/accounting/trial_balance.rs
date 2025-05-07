use async_graphql::{connection::*, *};

use lana_app::accounting::ledger_account::LedgerAccountChildrenCursor;

use crate::{
    graphql::loader::{LanaDataLoader, CHART_REF},
    primitives::*,
};

use super::{
    BtcLedgerAccountBalanceRange, LedgerAccount, LedgerAccountBalanceRangeByCurrency,
    UsdLedgerAccountBalanceRange,
};

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

    pub async fn accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<LedgerAccountChildrenCursor, LedgerAccount, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        crate::list_with_cursor!(
            LedgerAccountChildrenCursor,
            LedgerAccount,
            ctx,
            after,
            first,
            |query| app.accounting().list_account_children(
                sub,
                CHART_REF.0,
                self.entity.id,
                query,
                self.from.into_inner(),
                Some(self.until.into_inner()),
            )
        )
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
